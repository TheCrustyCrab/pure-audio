use std::f32::consts::TAU;
use pure_audio::{InstrumentAudioData, OutputBuffer, ParameterAutomationRate, ParameterDescriptor, ProcessorParameter};

#[derive(Copy, Clone)]
pub struct OscillatorFrequencyParameter(f32);

impl ProcessorParameter for OscillatorFrequencyParameter {
    const DESCRIPTOR: ParameterDescriptor = ParameterDescriptor {
        automation_rate: ParameterAutomationRate::K,
        default_value: 440.0,
        max_value: 2000.0,
        min_value: 20.0,
        name: "Frequency",
    };

    #[inline]
    fn from_parameter(value: f32) -> Self {
        OscillatorFrequencyParameter(value)
    }
}

#[derive(Default)]
pub struct OscillatorState {
    accumulator: u32,
    active: bool
}

pub fn process(
    InstrumentAudioData {
        events,
        outputs: OutputBuffer([[output]]),
        sample_rate,
        state: OscillatorState { accumulator, active },
    }: InstrumentAudioData<1, 1, 128, OscillatorState>,
    frequency: OscillatorFrequencyParameter,
) {
    for event in events {
        match event {
            pure_audio::Event::NoteOn => *active = true,
            pure_audio::Event::NoteOff => *active = false,
        }
    }

    if !*active {
        return;
    }

    for sample in output {
        *accumulator = accumulator.wrapping_add((frequency.0 / sample_rate * 10000.0) as u32);
        *sample = (TAU * *accumulator as f32 / 10000.0).sin();
    }
}
