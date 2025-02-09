use pure_audio::{
    parameter, AudioData, InputBuffer, OutputBuffer, SamplePrecise
};

#[parameter(text_to_value = volume_text_to_value, value_to_text = volume_value_to_text)]
pub struct Volume(f32);
    
#[inline]
fn volume_value_to_text(value: f64, writer: &mut impl std::fmt::Write) -> bool {
    let percentage = (value * 100.0).round();
    write!(writer, "{percentage}%").is_ok()
}

#[inline]
fn volume_text_to_value(text: &str) -> Option<f64> {
    match text.parse::<f64>() {
        Ok(parsed_value) => {
            Some(parsed_value / 100.0)
        },
        Err(_) => None
    }
}

pub fn process(
    AudioData {
        inputs: InputBuffer([[input]]),
        outputs: OutputBuffer([[output]]),
        ..
    }: AudioData,
    volume: Volume,
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
    volume: SamplePrecise<Volume>,
) {
    // only left side
    for ((input_sample, output_sample), volume) in input_l.iter().zip(output_l).zip(volume.values.iter()) {
        *output_sample = input_sample * volume;
    }
}

#[parameter(min = 0.0, max = 1.0, default = 0.5)]
pub struct Pan(f32);

pub fn process_stereo_gain_pan(
    AudioData {
        inputs: InputBuffer([[input_l, input_r]]),
        outputs: OutputBuffer([[output_l, output_r]]),
        ..
    }: AudioData<1, 1, 2, ()>,
    volume: SamplePrecise<Volume>,
    pan: SamplePrecise<Pan>
) {
    for (((input_sample, output_sample), volume), pan) in input_l.iter().zip(output_l).zip(volume.values.iter()).zip(pan.values.iter()) {
        *output_sample = input_sample * volume * (1.0 - pan);
    }

    for (((input_sample, output_sample), volume), pan) in input_r.iter().zip(output_r).zip(volume.values.iter()).zip(pan.values.iter()) {
        *output_sample = input_sample * volume * pan;
    }
}

