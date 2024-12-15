use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::AudioContext;
use pure_audio_wasm::PureAudioWorkletNode;

// factory-method called from the constructor of the worklet
#[wasm_bindgen(js_name = create_wasm_processor)]
pub fn create_gain_processor(sample_rate: f32) -> pure_audio_wasm::WasmProcessor {
    pure_audio_wasm::create_wasm_processor(gain::process, sample_rate)
}

// user-called method to create the node
#[wasm_bindgen]
pub async fn create_gain_node(ctx: &AudioContext) -> PureAudioWorkletNode {
    use wasm_bindgen::UnwrapThrowExt;

    pure_audio_wasm::register_and_create_node("Gain", gain::process, ctx).await.unwrap_throw()
}