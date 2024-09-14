use std::marker::PhantomData;
use pure_audio::{Event, IntoProcessor, ParameterDescriptor, Processor};
use wasm_bindgen::prelude::*;
use web_sys::AudioWorkletNode;
use crate::{InstrumentAudioWorkletNode, WasmAudioWorkletNode, PROCESSOR_BLOCK_LENGTH};

#[wasm_bindgen]
pub struct WasmProcessor {
    implementation: Box<dyn WasmProcessorImplementation>
}

impl WasmProcessor {
    fn new(implementation: Box<dyn WasmProcessorImplementation>) -> Self {
        Self {
            implementation
        }
    }
}

#[wasm_bindgen]
impl WasmProcessor {    
    pub fn get_inputs_ptr(&mut self) -> usize {
        self.implementation.get_inputs_ptr()
    }
    
    pub fn get_outputs_ptr(&self) -> usize {
        self.implementation.get_outputs_ptr()
    }

    pub fn get_parameters_ptr(&mut self) -> usize {
        self.implementation.get_parameters_ptr()
    }

    pub fn process(&mut self) {
        self.implementation.process();
    }

    pub fn note_on(&mut self, key: u8) {
        self.implementation.note_on(key);
    }

    pub fn note_off(&mut self, key: u8) {
        self.implementation.note_off(key);
    }
}

pub trait WasmProcessorImplementation: 'static {
    fn get_inputs_ptr(&mut self) -> usize;
    fn get_outputs_ptr(&self) -> usize;
    fn get_parameters_ptr(&mut self) -> usize;
    fn process(&mut self);
    fn note_on(&mut self, key: u8);
    fn note_off(&mut self, key: u8);
}

struct WasmProcessorWrapper<P, const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params> {
    processor: P,
    events: Vec<Event>,
    inputs: [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_INPUTS],
    outputs: [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS],
    parameters: [f32; NUM_PARAMS],
    marker: PhantomData<Params>
}

impl<P, const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, Params, const NUM_PARAMS: usize> WasmProcessorWrapper<P, IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
    fn new(processor: P) -> Self {
        Self {
            processor,
            events: vec![],
            inputs: [[[0.0; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_INPUTS],
            outputs: [[[0.0; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS],
            parameters: [0.0; NUM_PARAMS],
            marker: PhantomData
        }
    }
}

pub trait IntoWasmProcessor<const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> {
    type AudioWorkletNodeType: WasmAudioWorkletNode;
    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS];
    fn into_wasm_processor(self, sample_rate: f32) -> WasmProcessor;
}

pub trait IntoWasmProcessorImplementation<const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> {
    type AudioWorkletNodeType: WasmAudioWorkletNode;
    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS];
    fn into_wasm_processor_implementation(self, sample_rate: f32) -> impl WasmProcessorImplementation;
}

impl<I, const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> IntoWasmProcessor<IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S> for I
where
    I: IntoWasmProcessorImplementation<IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    type AudioWorkletNodeType = I::AudioWorkletNodeType;

    fn into_wasm_processor(self, sample_rate: f32) -> WasmProcessor {
        WasmProcessor::new(Box::new(self.into_wasm_processor_implementation(sample_rate)))
    }
    
    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS] {
        I::get_parameter_descriptors()
    }
}

impl<P, Params, const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize> WasmProcessorImplementation for WasmProcessorWrapper<P, IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
where
    P: 'static + Processor<IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, NUM_PARAMS, Params>,
    Params: 'static
{
    fn get_inputs_ptr(&mut self) -> usize {
        self.inputs.as_ptr() as *const _ as usize
    }

    fn get_outputs_ptr(&self) -> usize {
        self.outputs.as_ptr() as *const _ as usize
    }

    fn get_parameters_ptr(&mut self) -> usize {
        self.parameters.as_ptr() as *const _ as usize
    }

    fn process(&mut self) {
        // clear outputs
        self.outputs = [[[0.0; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS];
        self.processor.process(&self.inputs, &mut self.outputs, &self.parameters, &self.events);
        self.events.clear();
    }

    fn note_on(&mut self, key: u8) {
        self.events.push(Event::NoteOn { key });
    }

    fn note_off(&mut self, key: u8) {
        self.events.push(Event::NoteOff { key });
    }
}

impl<F, Params, const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, S> IntoWasmProcessorImplementation<IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S> for F
where 
    F: 'static + IntoProcessor<IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, NUM_PARAMS, Params, S> + AudioWorkletNodeType<IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>,
    Params: 'static,
    S: 'static + Default
{
    type AudioWorkletNodeType = F::AudioWorkletNodeType;

    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS] {
        F::get_parameter_descriptors()
    }

    fn into_wasm_processor_implementation(self, sample_rate: f32) -> impl WasmProcessorImplementation {
        WasmProcessorWrapper::new(self.into_processor(sample_rate))
    }
}

pub trait AudioWorkletNodeType<const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> {
    type AudioWorkletNodeType: WasmAudioWorkletNode;
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> AudioWorkletNodeType<false, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S> for P
where 
    P: IntoProcessor<false, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, NUM_PARAMS, Params, S>
{
    type AudioWorkletNodeType = AudioWorkletNode;
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> AudioWorkletNodeType<true, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S> for P
where 
    P: IntoProcessor<true, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, NUM_PARAMS, Params, S>
{
    type AudioWorkletNodeType = InstrumentAudioWorkletNode;
}