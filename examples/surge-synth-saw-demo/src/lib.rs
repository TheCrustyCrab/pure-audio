use core::f32;
use std::{array, fmt::Write};
use pure_audio::{Event, OutEvent, State, StereoSynthData, parameter};
use random::get_random;
use voice::Voice;

mod random;
mod voice;
mod tests;

const MAX_VOICES: usize = 64;
const MAX_UNISON: usize = 7;

#[parameter(min = 0, max = 7, default = 3, value_to_text = uni_value_to_text)]
struct UnisonCount(u32);

fn uni_value_to_text(value: f64, writer: &mut impl Write) -> bool {
    write!(writer, "{value} voices").is_ok()
} 

#[parameter(min = 0, max = 100, default = 10)]
struct UnisonSpread(f32);

#[parameter(min = -200, max = 200, default = 0)]
struct OscillatorDetune(f32);

#[parameter(min = 0, max = 1, default = 0.01)]
struct AmplitudeAttack(f32);

#[parameter(min = 0, max = 1, default = 0.2)]
struct AmplitudeRelease(f32);

#[parameter(default = false)]
struct AmplitudeEnvelopeIsGate(bool);

#[parameter(min = 0, max = 1, default = 1)]
struct PreFilterVCA(f32);

#[parameter(min = 1, max = 127, default = 69)]
struct Cutoff(f32);

#[parameter(min = 0, max = 1, default = 0.7)]
struct Resonance(f32);

#[parameter(default = "HighPass")]
#[derive(PartialEq)]
enum FilterMode {
    LowPass,
    HighPass,
    BandPass,
    Notch,
    Peak,
    All,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum AEGMode {
    #[default]
    Off,
    Attack,
    Hold,
    NewlyOff,
    Releasing,
}

struct SawState {
    voices: [Voice; MAX_VOICES]
}

impl Default for SawState {
    fn default() -> Self {
        Self {
            voices: array::from_fn(|_| Voice::default())
        }
    }
}

impl State for SawState {
    fn activate(&mut self, sample_rate: f64, _min_frame_count: usize, _max_frame_count: usize) {
        for voice in self.voices.iter_mut() {
            voice.sample_rate = sample_rate;
        }
    }
}

fn process(
    StereoSynthData {
        events,
        out_events,
        outputs: [output_l, output_r],
        state: SawState { voices },
    }: StereoSynthData<SawState>,
    unison_count: UnisonCount,
    unison_spread: UnisonSpread,
    oscillator_detune: OscillatorDetune,
    amplitude_attack: AmplitudeAttack,
    amplitude_release: AmplitudeRelease,
    amplitude_envelope_is_gate: AmplitudeEnvelopeIsGate,
    prefilter_vca: PreFilterVCA,
    cutoff: Cutoff,
    resonance: Resonance,
    filter_mode: FilterMode,
) {
    for event in events {
        match event {
            &Event::NoteOn {
                port_index,
                channel,
                key,
                note_id,
                ..
            } => {
                let voice = voices
                    .iter_mut()
                    .find(|voice| voice.aeg_mode == AEGMode::Off);
                if let Some(voice) = voice {
                    voice.activate(
                        port_index,
                        channel,
                        key as i32,
                        note_id,
                        filter_mode,
                        unison_count,
                        unison_spread,
                        oscillator_detune,
                        cutoff,
                        resonance,
                        prefilter_vca,
                        amplitude_attack,
                        amplitude_release,
                        amplitude_envelope_is_gate,
                    );
                } else {
                    let index = get_random(MAX_VOICES);
                    let voice = &mut voices[index];
                    out_events.push(OutEvent::NoteEnd {
                        port_index: voice.port_id,
                        channel: voice.channel,
                        key: voice.key as u8,
                        note_id: voice.note_id,
                        velocity: 0,
                    });
                    voice.activate(
                        port_index,
                        channel,
                        key as i32,
                        note_id,
                        filter_mode,
                        unison_count,
                        unison_spread,
                        oscillator_detune,
                        cutoff,
                        resonance,
                        prefilter_vca,
                        amplitude_attack,
                        amplitude_release,
                        amplitude_envelope_is_gate,
                    );
                }
            }
            &Event::NoteOff {
                port_index,
                channel,
                key,
                ..
            } => {
                let voices = voices.iter_mut().filter(|voice| {
                    voice.is_playing()
                        && voice.key == key as i32
                        && voice.port_id == port_index
                        && voice.channel == channel
                });
                for voice in voices {
                    voice.release();
                }
            }
            Event::ParamsChanged => {
                for voice in voices.iter_mut().filter(|voice| voice.is_playing()) {
                    voice.update(
                        unison_spread,
                        oscillator_detune,
                        cutoff,
                        resonance,
                        prefilter_vca,
                        amplitude_attack,
                        amplitude_release,
                        amplitude_envelope_is_gate,
                        filter_mode,
                    );
                }
            }
        }
    }

    for (sample_l, sample_r) in output_l.iter_mut().zip(output_r) {
        (*sample_l, *sample_r) = (0.0, 0.0);

        for voice in voices.iter_mut().filter(|voice| voice.is_playing()) {
            voice.step();
            *sample_l += voice.out_l;
            *sample_r += voice.out_r;
        }
    }

    for Voice {
        aeg_mode,
        port_id,
        channel,
        key,
        note_id,
        ..
    } in voices
        .iter_mut()
        .filter(|voice| voice.aeg_mode == AEGMode::NewlyOff)
    {
        out_events.push(OutEvent::NoteEnd {
            port_index: *port_id,
            channel: *channel,
            key: *key as u8,
            note_id: *note_id,
            velocity: 0,
        });
        *aeg_mode = AEGMode::Off;
    }
}

#[cfg(target_arch = "wasm32")]
pure_audio_wasm::pure_audio_wasm_entry!("SurgeSynthSaw", process);

#[cfg(not(target_arch = "wasm32"))]
pure_audio_clap::pure_audio_clap_entry!("pureaudio.SurgeSynthSaw", "SurgeSynthSaw", process);
