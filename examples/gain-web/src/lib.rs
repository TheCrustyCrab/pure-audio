use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{AudioContext, AudioWorkletNode};

// factory-method called from the constructor of the worklet
#[cfg(feature = "build_processor")]
#[wasm_bindgen(js_name = create_wasm_processor)]
pub fn create_gain_processor(sample_rate: f32) -> pure_audio_wasm::WasmEffectProcessor { // todo: 1 type
    pure_audio_wasm::create_wasm_processor(gain::process, sample_rate)
}

// user-called method to create the node
#[cfg(not(feature = "build_processor"))]
#[wasm_bindgen]
pub async fn create_gain_node(ctx: &AudioContext) -> AudioWorkletNode {
    pure_audio_wasm::register_and_create_node("Gain", gain::process, ctx).await.unwrap()
}