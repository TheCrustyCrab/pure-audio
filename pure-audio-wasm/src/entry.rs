#[macro_export]
macro_rules! pure_audio_wasm_entry {
    ($name:ident, $process:expr) => {
        const _: () = {
            use pure_audio_wasm::paste;
            use pure_audio_wasm::paste::paste;
            use pure_audio_wasm::wasm_bindgen;
            use pure_audio_wasm::wasm_bindgen_futures;
            use pure_audio_wasm::{wasm_bindgen::{prelude::wasm_bindgen, UnwrapThrowExt}, AudioContext, PureAudioWorkletNode};

            paste! { 
                #[wasm_bindgen(js_name = [<create_ $name:lower _wasm_processor>])]
                pub fn create_wasm_processor(sample_rate: f32) -> pure_audio_wasm::WasmProcessor {
                    pure_audio_wasm::create_wasm_processor($process, sample_rate)
                }
                
                #[wasm_bindgen]
                pub async fn [<create_ $name:lower _node>](ctx: &AudioContext) -> PureAudioWorkletNode {
                    pure_audio_wasm::register_and_create_node(stringify!([<$name:lower>]), $process, ctx).await.unwrap_throw()
                }
            }
        };
    };
}