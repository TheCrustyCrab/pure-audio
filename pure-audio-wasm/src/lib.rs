mod audio_worklet_node;
mod entry;
mod es_module;
mod event;
mod loader;
mod processor;

// re-export
pub use audio_worklet_node::*;
pub use js_sys;
pub use loader::*;
pub use processor::*;
use pure_audio::IntoProcessor;
pub use wasm_bindgen;
pub use wasm_bindgen_futures;
pub use web_sys::AudioContext;

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
    sample_rate: f32,
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
