use std::{collections::HashMap, f32::consts::TAU};
use pure_audio::{InstrumentAudioData, OutputBuffer};

struct Voice {
    phase: u32,
    velocity: u8 // 0-127
}

#[derive(Default)]
pub struct OscillatorState {
    active: bool,
    voices: HashMap<u8, Voice>
}

pub fn process(
    InstrumentAudioData {
        events,
        outputs: OutputBuffer([[output]]),
        sample_rate,
        state: OscillatorState { active, voices },
    }: InstrumentAudioData<1, 1, 128, OscillatorState>
) {
    for event in events {
        match event {
            pure_audio::Event::NoteOn { key, velocity } => {
                *active = true;
                voices.insert(*key, Voice { phase: 0, velocity: *velocity });
            },
            pure_audio::Event::NoteOff { key, .. } => {
                *active = false;
                voices.remove(key);
            },
        }
    }

    if !*active {
        return;
    }

    for sample in output {
        let mut sum = 0.0;
        let gain_per_voice = 1.0 / voices.len() as f32;
        for (key, Voice { phase, velocity }) in voices.iter_mut() {
            let freq = 440.0 * 2f32.powf((*key as f32 - 57.0) / 12.0);
            let velocity_gain = *velocity as f32 / 127.0;
            *phase = phase.wrapping_add((freq / sample_rate * 10000.0) as u32);
            sum += (TAU * *phase as f32 / 10000.0).sin() * velocity_gain * gain_per_voice;
        }
        *sample = sum;
    }
}
