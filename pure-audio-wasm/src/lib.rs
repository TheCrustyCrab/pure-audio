mod audio_worklet_node;
mod entry;
mod es_module;
mod event;
mod loader;
mod parameter;
mod processor;

// re-export
pub use audio_worklet_node::*;
pub use js_sys;
pub use loader::*;
use parameter::{ParameterConverter, WasmParameterConverter};
pub use processor::*;
use pure_audio::IntoProcessor;
pub use wasm_bindgen;
use wasm_bindgen::prelude::*;
pub use wasm_bindgen_futures;
pub use web_sys;

// https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Using_AudioWorklet#the_input_and_output_lists
// currently fixed size
// when dynamic: allocate sufficient space and use the required amount
const PROCESSOR_BLOCK_LENGTH: usize = 128;

pub fn create_wasm_processor<
    P,
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    A,
    Params,
    S,
>(
    process: P,
    sample_rate: f64,
    output_event_callback: js_sys::Function,
) -> WasmProcessor
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
    A: 'static,
    Params: 'static,
    S: 'static + Default,
{
    process.into_wasm_processor(sample_rate, output_event_callback)
}

pub fn create_raw_wasm_parameter_converter<
    P,
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    A,
    Params,
    S,
>(
    _process: P,
) -> usize
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
    A: 'static,
    Params: 'static,
    S: 'static + Default,
{
    let implementation = ParameterConverter::<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>::new();
    WasmParameterConverter::new(Box::new(implementation))
}

pub fn value_to_text(ptr: usize, index: usize, value: f64) -> JsValue {
    let parameter_converter = WasmParameterConverter::from_raw_ptr(ptr);
    parameter_converter.value_to_text(index, value).into()
}

pub fn destroy_raw_wasm_parameter_converter(ptr: usize) {
    WasmParameterConverter::destroy(ptr);
}
