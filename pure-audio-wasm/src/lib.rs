mod es_module;
mod loader;
mod processor;

// re-export
pub use loader::*;
pub use processor::*;

// https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Using_AudioWorklet#the_input_and_output_lists
// currently fixed size
// when dynamic: allocate sufficient space and use the required amount
const PROCESSOR_BLOCK_LENGTH: usize = 128;

pub fn create_wasm_processor<
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    Params,
    P: WasmProcessor,
    S,
>(
    process: impl IntoWasmProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, P, S>,
    sample_rate: f32,
) -> P {
    process.into_wasm_processor(sample_rate)
}