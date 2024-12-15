use wasm_bindgen::{prelude::wasm_bindgen, UnwrapThrowExt};
use web_sys::AudioContext;
use pure_audio_wasm::PureAudioWorkletNode;

// todo: macro to generate both entrypoints
// pure_audio_wasm_entry!("Oscillator", oscillator::process)

// factory-method called from the constructor of the worklet
#[wasm_bindgen(js_name = create_wasm_processor)]
pub fn create_oscillator_processor(sample_rate: f32) -> pure_audio_wasm::WasmProcessor {
    pure_audio_wasm::create_wasm_processor(oscillator::process, sample_rate)
}

// user-called method to create the node
#[wasm_bindgen]
pub async fn create_oscillator_node(ctx: &AudioContext) -> PureAudioWorkletNode {
    pure_audio_wasm::register_and_create_node("Oscillator", oscillator::process, ctx).await.unwrap_throw()
}
