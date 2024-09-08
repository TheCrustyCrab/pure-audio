use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::AudioContext;
use pure_audio_wasm::InstrumentAudioWorkletNode;

// factory-method called from the constructor of the worklet
#[cfg(feature = "build_processor")]
#[wasm_bindgen(js_name = create_wasm_processor)]
pub fn create_gain_processor(sample_rate: f32) -> pure_audio_wasm::WasmProcessor {
    pure_audio_wasm::create_wasm_processor(oscillator::process, sample_rate)
}

// user-called method to create the node
#[cfg(not(feature = "build_processor"))]
#[wasm_bindgen]
pub async fn create_oscillator_node(ctx: &AudioContext) -> InstrumentAudioWorkletNode {
    pure_audio_wasm::register_and_create_node("Oscillator", oscillator::process, ctx).await.unwrap()
}
