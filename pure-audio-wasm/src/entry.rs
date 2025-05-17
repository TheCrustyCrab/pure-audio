#[macro_export]
macro_rules! pure_audio_wasm_entry {
    ($name:expr, $process:expr) => {
        const _: () = {
            use pure_audio_wasm::js_sys;
            use pure_audio_wasm::wasm_bindgen;
            use pure_audio_wasm::wasm_bindgen::JsValue;
            use pure_audio_wasm::wasm_bindgen_futures;
            use pure_audio_wasm::{wasm_bindgen::{prelude::wasm_bindgen, UnwrapThrowExt}, AudioContext, PureAudioWorkletNode};

            #[wasm_bindgen(js_name = createWasmProcessor)]
            pub fn create_wasm_processor(sample_rate: f32, output_event_callback: js_sys::Function) -> pure_audio_wasm::WasmProcessor {
                pure_audio_wasm::create_wasm_processor($process, sample_rate, output_event_callback)
            }
            
            #[wasm_bindgen(js_name = createAudioNode)]
            pub async fn create_node(ctx: &AudioContext) -> PureAudioWorkletNode {
                pure_audio_wasm::register_and_create_node($name, $process, ctx, None).await.unwrap_throw()
            }
            
            #[wasm_bindgen(js_name = createAudioNodeWithGeneratedParameterUI)]
            pub async fn create_node_with_generated_parameter_ui(ctx: &AudioContext, #[wasm_bindgen(js_name = divId)] div_id: &str) -> PureAudioWorkletNode {
                pure_audio_wasm::register_and_create_node($name, $process, ctx, Some(div_id)).await.unwrap_throw()
            }

            #[wasm_bindgen(js_name = createRawWasmParameterConverter)]
            pub fn create_raw_wasm_parameter_converter() -> usize {
                pure_audio_wasm::create_raw_wasm_parameter_converter($process)
            }

            #[wasm_bindgen(js_name = destroyRawWasmParameterConverter)]
            pub fn destroy_raw_wasm_parameter_converter(ptr: usize) {
                pure_audio_wasm::destroy_raw_wasm_parameter_converter(ptr)
            }

            #[wasm_bindgen(js_name = valueToText)]
            pub fn value_to_text(ptr: usize, index: usize, value: f64) -> JsValue {
                pure_audio_wasm::value_to_text(ptr, index, value)
            }
        };
    };
}