use pure_audio::{parameter, SamplePrecise, StereoEffectData};

#[parameter(min = -10, max = 10, default = 0)]
pub struct Pan(i32);

pub fn process(
    StereoEffectData {
        inputs: [input_l, input_r],
        outputs: [output_l, output_r],
        ..
    }: StereoEffectData,
    pan: SamplePrecise<Pan>
) {
    for ((((input_sample_l, output_sample_l), input_sample_r), output_sample_r), pan) in input_l.iter().zip(output_l).zip(input_r).zip(output_r).zip(pan.values()) {
        let pan = (pan.0 as f32 + 10.0) / 20.0;
        let pan_l = 1.0 - pan;
        *output_sample_l = input_sample_l * pan_l;
        *output_sample_r = input_sample_r * pan;
    }
}

#[cfg(target_arch = "wasm32")]
pure_audio_wasm::pure_audio_wasm_entry!("Pan", process);

#[cfg(not(target_arch = "wasm32"))]
pure_audio_clap::pure_audio_clap_entry!("pureaudio.Pan", "PureAudioPan", process);