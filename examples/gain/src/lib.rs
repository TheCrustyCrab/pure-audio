use pure_audio::{parameter, MonoEffectData};

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
        Ok(parsed_value) => Some(parsed_value / 100.0),
        Err(_) => None,
    }
}

pub fn process(MonoEffectData { input, output, .. }: MonoEffectData, volume: Volume) {
    for (input_sample, output_sample) in input.iter().zip(output) {
        *output_sample = input_sample * volume;
    }
}

#[cfg(target_arch = "wasm32")]
pure_audio_wasm::pure_audio_wasm_entry!("Gain", process);

#[cfg(not(target_arch = "wasm32"))]
pure_audio_clap::pure_audio_clap_entry!("pureaudio.Gain", "PureAudioGain", process);
