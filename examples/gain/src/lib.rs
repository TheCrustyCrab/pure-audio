use pure_audio::{
    AudioData, InputBuffer, OutputBuffer, ParameterAutomationRate, ParameterDescriptor,
    ProcessorParameter,
};
use std::ops::Mul;

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
    
    #[inline]
    fn value_to_text(value: f64, writer: &mut impl std::fmt::Write) -> bool {
        let percentage = (value * 100.0).round();
        write!(writer, "{percentage}%").is_ok()
    }
    
    #[inline]
    fn text_to_value(text: &str) -> Option<f64> {
        match text.parse::<f64>() {
            Ok(parsed_value) => {
                Some(parsed_value / 100.0)
            },
            Err(_) => None
        }
    }
}

// todo: operator implementations for ProcessorParameter
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
    AudioData {
        inputs: InputBuffer([[input]]),
        outputs: OutputBuffer([[output]]),
        ..
    }: AudioData,
    volume: GainVolumeParameter,
) {
    for (input_sample, output_sample) in input.iter().zip(output) {
        *output_sample = input_sample * volume;
    }
}

pub fn process_stereo(
    AudioData {
        inputs: InputBuffer([[input_l, input_r]]),
        outputs: OutputBuffer([[output_l, output_r]]),
        ..
    }: AudioData<1, 1, 2, ()>,
    volume: GainVolumeParameter,
) {
    // only left side
    for (input_sample, output_sample) in input_l.iter().zip(output_l) {
        *output_sample = input_sample * volume;
    }
}
