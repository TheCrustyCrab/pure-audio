use std::ops::Mul;
use pure_audio::{EffectAudioData, InputBuffer, OutputBuffer, ParameterAutomationRate, ParameterDescriptor, ProcessorParameter};

#[derive(Copy, Clone)]
pub struct GainVolumeParameter(f32);

impl ProcessorParameter for GainVolumeParameter {
    const DESCRIPTOR: ParameterDescriptor = ParameterDescriptor {
        automation_rate: ParameterAutomationRate::K,
        default_value: 1.0,
        max_value: 1.0,
        min_value: 0.0,
        name: "Volume",
    };

    #[inline]
    fn from_parameter(value: f32) -> Self {
        GainVolumeParameter(value)
    }
}

impl Mul<f32> for GainVolumeParameter {
    type Output = f32;

    fn mul(self, rhs: f32) -> Self::Output {
        self.0 * rhs
    }
}

impl Mul<&f32> for GainVolumeParameter {
    type Output = f32;

    fn mul(self, rhs: &f32) -> Self::Output {
        self.0 * rhs
    }
}

impl Mul<GainVolumeParameter> for f32 {
    type Output = f32;

    fn mul(self, rhs: GainVolumeParameter) -> Self::Output {
        self * rhs.0
    }
}

impl Mul<GainVolumeParameter> for &f32 {
    type Output = f32;

    fn mul(self, rhs: GainVolumeParameter) -> Self::Output {
        self * rhs.0
    }
}

pub fn process(
    EffectAudioData {
        inputs: InputBuffer([[ref input]]),
        outputs: OutputBuffer([[output]]),
        ..
    }: EffectAudioData,
    volume: GainVolumeParameter,
) {
    for (input_sample, output_sample) in input.iter().zip(output) {
        *output_sample = input_sample * volume;
    }
}