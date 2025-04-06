use std::{collections::HashMap, f32::consts::TAU};
use pure_audio::MonoSynthData;

struct Voice {
    phase: f32,
    phase_increment: f32,
    velocity_gain: f32
}

impl Voice {
    #[inline]
    fn new(key: u8, velocity: u8 /* 0-127 */, sample_rate: f32) -> Self {
        let frequency = 440.0 * 2f32.powf((key as f32 - 57.0) / 12.0);
        let phase_increment = frequency / sample_rate;
        let velocity_gain = velocity as f32 / 127.0;
        Self {
            phase: 0.0,
            phase_increment,
            velocity_gain
        }
    }

    #[inline]
    fn advance(&mut self) -> f32 {
        self.phase += self.phase_increment;
        // avoid overflow on phase
        // sin(1*2pi) = sin(0*2pi) = 0
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }
        self.phase
    }
}

#[derive(Default)]
pub struct OscillatorState {
    voices: HashMap<u8, Voice>
}

pub fn process(
    MonoSynthData {
        output,
        events,
        sample_rate,
        state: OscillatorState { voices }
    }: MonoSynthData<OscillatorState>
) {
    for event in events {
        match event {
            &pure_audio::Event::NoteOn { key, velocity } => {
                voices.insert(key, Voice::new(key, velocity, sample_rate));
            },
            pure_audio::Event::NoteOff { key, .. } => {
                voices.remove(key);
            },
            _ => {}
        }
    }

    let gain_per_voice = 1.0 / voices.len() as f32;

    for sample in output {
        let sum = 
            voices
                .values_mut()
                .fold(0.0f32, |current, voice| 
                    current + (TAU * voice.advance()).sin() * voice.velocity_gain * gain_per_voice);
        *sample = sum;
    }
}

#[cfg(target_arch = "wasm32")]
pure_audio_wasm::pure_audio_wasm_entry!(Oscillator, process);

#[cfg(not(target_arch = "wasm32"))]
pure_audio_clap::pure_audio_clap_entry!("pureaudio.Oscillator", "PureAudioOscillator", process);