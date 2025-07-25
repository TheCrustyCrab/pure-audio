mod tests;

use pure_audio::{parameter, Event, IsPlaying, MonoEffectData, State, Tempo};
use std::f32::consts::{FRAC_2_PI, FRAC_PI_2, TAU};

#[parameter(variant_display_names = ["1/1", "1/2", "1/4", "1/8", "1/16"])]
enum Rate {
    One,
    Second,
    Quarter,
    Eighth,
    Sixteenth,
}

#[parameter(min = 0.001, max = 1, default = 1)]
struct Depth(f32);

#[parameter]
enum Shape {
    Sine,
    Triangle
}

struct LFO {
    rate: Rate,
    depth_denominator: f32,
    phase: f64,
    phase_increment: f64,
    sample_rate: f64,
    host_is_playing: bool,
    tempo: f32,
}

impl Default for LFO {
    fn default() -> Self {
        let mut lfo = Self {
            rate: Rate::One,
            depth_denominator: 0.0,
            phase: 0.0,
            phase_increment: 0.0,
            sample_rate: 0.0,
            host_is_playing: false,
            tempo: 0.0,
        };
        lfo.set_depth(1.0);
        lfo
    }
}

impl State for LFO {
    fn activate(&mut self, sample_rate: f64, _min_frame_count: usize, _max_frame_count: usize) {
        self.set_sample_rate(sample_rate);
    }
}

impl LFO {
    fn advance(&mut self) -> f64 {
        self.phase += self.phase_increment;
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }
        self.phase
    }

    fn set_rate(&mut self, value: Rate) {
        self.rate = value;
        self.recalculate();
    }

    fn set_depth(&mut self, value: f32) {
        self.depth_denominator = 2.0 * (1.0 / value);
    }

    fn set_sample_rate(&mut self, value: f64) {
        self.sample_rate = value;
        self.recalculate();
    }

    fn set_is_playing(&mut self, value: bool) {
        self.host_is_playing = value;
        if value {
            self.phase = 0.0; // reset phase to ensure beat/peak alignment
        }
    }

    fn set_tempo(&mut self, value: f32) {
        self.tempo = value;
        self.recalculate();
    }

    fn recalculate(&mut self) {
        let multiplier = match self.rate {
            Rate::One => 1.0,
            Rate::Second => 2.0,
            Rate::Quarter => 4.0,
            Rate::Eighth => 8.0,
            Rate::Sixteenth => 16.0,
        };
        let frequency = self.tempo / 60.0 * multiplier;
        self.phase_increment = frequency as f64 / self.sample_rate;
    }
}

fn process(
    MonoEffectData {
        input,
        output,
        state: lfo,
        events,
        ..
    }: MonoEffectData<LFO>,
    IsPlaying(is_playing): IsPlaying,
    Tempo(tempo): Tempo,
    rate: Rate,
    Depth(depth): Depth,
    shape: Shape,
) {
    if lfo.host_is_playing != is_playing {
        lfo.set_is_playing(is_playing);
    }

    if lfo.tempo != tempo {
        lfo.set_tempo(tempo);
    }

    for event in events {
        if let Event::ParamsChanged = event {
            lfo.set_rate(rate);
            lfo.set_depth(depth);
        }
    }

    for (input_sample, output_sample) in input.iter().zip(output) {
        let gain = match shape {
            // normalize to [1-depth, 1] + shift phase by pi/2 to start with a peak
            Shape::Sine => 1.0 - ((TAU * lfo.advance() as f32 - FRAC_PI_2).sin() + 1.0) / lfo.depth_denominator,
            Shape::Triangle => 1.0 - (FRAC_2_PI * ((TAU * lfo.advance() as f32 - FRAC_PI_2).sin()).asin() + 1.0) / lfo.depth_denominator,
        };
        *output_sample = input_sample * gain;
    }
}

#[cfg(target_arch = "wasm32")]
pure_audio_wasm::pure_audio_wasm_entry!("Tremolo", process);

#[cfg(not(target_arch = "wasm32"))]
pure_audio_clap::pure_audio_clap_entry!("pureaudio.Tremolo", "PureAudioTremolo", process);
