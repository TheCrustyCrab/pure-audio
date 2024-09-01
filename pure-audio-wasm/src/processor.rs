use std::marker::PhantomData;
use pure_audio::{EffectAudioData, FromParameters, InputBuffer, InstrumentAudioData, OutputBuffer, ParameterDescriptor};
use wasm_bindgen::prelude::*;
use web_sys::AudioWorkletNode;
use crate::{InstrumentAudioWorkletNode, WasmAudioWorkletNode, PROCESSOR_BLOCK_LENGTH};

pub trait WasmProcessor {
    type AudioWorkletNodeType: WasmAudioWorkletNode;
}

#[wasm_bindgen]
pub struct WasmEffectProcessor {
    implementation: Box<dyn EffectProcessorImplementation>
}

impl WasmEffectProcessor {
    fn new(implementation: Box<dyn EffectProcessorImplementation>) -> Self {
        Self {
            implementation
        }
    }
}

impl WasmProcessor for WasmEffectProcessor {
    type AudioWorkletNodeType = AudioWorkletNode;
}

#[wasm_bindgen]
impl WasmEffectProcessor {    
    pub fn get_inputs_ptr(&mut self) -> usize {
        self.implementation.get_inputs_ptr()
    }
    
    pub fn get_outputs_ptr(&self) -> usize {
        self.implementation.get_outputs_ptr()
    }

    pub fn get_parameters_ptr(&mut self) -> usize {
        self.implementation.get_parameters_ptr()
    }

    pub unsafe fn process(&mut self) {
        self.implementation.process();
    }
}

#[wasm_bindgen]
pub struct WasmInstrumentProcessor {
    implementation: Box<dyn InstrumentProcessorImplementation>
}

impl WasmInstrumentProcessor {
    fn new(implementation: Box<dyn InstrumentProcessorImplementation>) -> Self {
        Self {
            implementation
        }
    }
}

impl WasmProcessor for WasmInstrumentProcessor {
    type AudioWorkletNodeType = InstrumentAudioWorkletNode;
}

#[wasm_bindgen]
impl WasmInstrumentProcessor {    
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
}

pub trait EffectProcessorImplementation: 'static {
    fn get_inputs_ptr(&mut self) -> usize;
    fn get_outputs_ptr(&self) -> usize;
    fn get_parameters_ptr(&mut self) -> usize;
    fn process(&mut self);
}

pub trait InstrumentProcessorImplementation: 'static {
    fn get_inputs_ptr(&mut self) -> usize;
    fn get_outputs_ptr(&self) -> usize;
    fn get_parameters_ptr(&mut self) -> usize;
    fn process(&mut self);
}

struct ProcessorWrapper<F, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> {
    f: F,
    inputs: [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_INPUTS],
    outputs: [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS],
    parameters: [f32; NUM_PARAMS],
    sample_rate: f32,
    state: S,
    marker: PhantomData<Params>
}

impl<F, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, Params, S, const NUM_PARAMS: usize> ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    fn new(f: F, sample_rate: f32, state: S) -> Self {
        Self {
            f,
            inputs: [[[0.0; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_INPUTS],
            outputs: [[[0.0; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS],
            parameters: [0.0; NUM_PARAMS],
            sample_rate,
            state,
            marker: PhantomData
        }
    }
}

pub trait IntoWasmProcessor<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, P: WasmProcessor, S> {
    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS];
    fn into_wasm_processor(self, sample_rate: f32) -> P;
}

pub trait IntoEffectProcessorImplementation<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> {
    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS];
    fn into_effect_processor_implementation(self, sample_rate: f32) -> impl EffectProcessorImplementation;
}

pub(self) trait IntoInstrumentProcessorImplementation<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> {
    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS];
    fn into_instrument_processor_implementation(self, sample_rate: f32) -> impl InstrumentProcessorImplementation;
}

impl<T, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> IntoWasmProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, WasmEffectProcessor, S> for T
where
    T: IntoEffectProcessorImplementation<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    fn into_wasm_processor(self, sample_rate: f32) -> WasmEffectProcessor {
        WasmEffectProcessor::new(Box::new(self.into_effect_processor_implementation(sample_rate)))
    }
    
    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS] {
        T::get_parameter_descriptors()
    }
}

impl<T, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S> IntoWasmProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, WasmInstrumentProcessor, S> for T
where
    T: IntoInstrumentProcessorImplementation<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    fn into_wasm_processor(self, sample_rate: f32) -> WasmInstrumentProcessor {
        WasmInstrumentProcessor::new(Box::new(self.into_instrument_processor_implementation(sample_rate)))
    }
    
    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS] {
        T::get_parameter_descriptors()
    }
}

// effect with 1 parameter
impl<F, P1, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S> EffectProcessorImplementation for ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 1, (P1,), S>
where 
    F: 'static + FnMut(EffectAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default
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
        let p1 = P1::from_parameters(&self.parameters, 0);
        let data = EffectAudioData {
            inputs: InputBuffer::new(&self.inputs),
            outputs: OutputBuffer::new(&mut self.outputs),
            sample_rate: self.sample_rate,
            state: &mut self.state
        };
        (self.f)(data, p1);
    }
}

impl<F, P1, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S> IntoEffectProcessorImplementation<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 1, (P1,), S> for F
where 
    F: 'static + FnMut(EffectAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default
{
    fn get_parameter_descriptors() -> [ParameterDescriptor; 1] {
        [P1::DESCRIPTOR]
    }

    fn into_effect_processor_implementation(self, sample_rate: f32) -> impl EffectProcessorImplementation {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
}

// effect with 2 parameters
impl<F, P1, P2, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S> EffectProcessorImplementation for ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 2, (P1, P2), S>
where
    F: 'static + FnMut(EffectAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, S>, P1, P2),
    P1: 'static + FromParameters,
    P2: 'static + FromParameters,
    S: 'static + Default
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
        let p1 = P1::from_parameters(&self.parameters, 0);
        let p2 = P2::from_parameters(&self.parameters, 1);
        let data = EffectAudioData {
            inputs: InputBuffer::new(&self.inputs),
            outputs: OutputBuffer::new(&mut self.outputs),
            sample_rate: self.sample_rate,
            state: &mut self.state
        };
        (self.f)(data, p1, p2);
    }
}

impl<F, P1, P2, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S> IntoEffectProcessorImplementation<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 2, (P1, P2), S> for F
where 
    F: 'static + FnMut(EffectAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, S>, P1, P2),
    P1: 'static + FromParameters,
    P2: 'static + FromParameters,
    S: 'static + Default
{
    fn get_parameter_descriptors() -> [ParameterDescriptor; 2] {
        [P1::DESCRIPTOR, P2::DESCRIPTOR]
    }

    fn into_effect_processor_implementation(self, sample_rate: f32) -> impl EffectProcessorImplementation {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
}

// instrument with 1 parameter
impl<F, P1, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S> InstrumentProcessorImplementation for ProcessorWrapper<F, 0, NUM_OUTPUTS, NUM_CHANNELS, 1, (P1,), S>
where 
    F: 'static + FnMut(InstrumentAudioData<NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default
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
        let p1 = P1::from_parameters(&self.parameters, 0);
        let data = InstrumentAudioData {
            outputs: OutputBuffer::new(&mut self.outputs),
            sample_rate: self.sample_rate,
            state: &mut self.state
        };
        (self.f)(data, p1);
    }
}

impl<F, P1, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S> IntoInstrumentProcessorImplementation<0, NUM_OUTPUTS, NUM_CHANNELS, 1, (P1,), S> for F
where 
    F: 'static + FnMut(InstrumentAudioData<NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default
{
    fn get_parameter_descriptors() -> [ParameterDescriptor; 1] {
        [P1::DESCRIPTOR]
    }

    fn into_instrument_processor_implementation(self, sample_rate: f32) -> impl InstrumentProcessorImplementation {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
}

// instrument with 2 parameters
impl<F, P1, P2, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S> InstrumentProcessorImplementation for ProcessorWrapper<F, 0, NUM_OUTPUTS, NUM_CHANNELS, 2, (P1, P2), S>
where 
    F: 'static + FnMut(InstrumentAudioData<NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, S>, P1, P2),
    P1: 'static + FromParameters,
    P2: 'static + FromParameters,
    S: 'static + Default
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
        let p1 = P1::from_parameters(&self.parameters, 0);
        let p2 = P2::from_parameters(&self.parameters, 1);
        let data = InstrumentAudioData {
            outputs: OutputBuffer::new(&mut self.outputs),
            sample_rate: self.sample_rate,
            state: &mut self.state
        };
        (self.f)(data, p1, p2);
    }
}

impl<F, P1, P2, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S> IntoInstrumentProcessorImplementation<0, NUM_OUTPUTS, NUM_CHANNELS, 2, (P1, P2), S> for F
where 
    F: 'static + FnMut(InstrumentAudioData<NUM_OUTPUTS, NUM_CHANNELS, PROCESSOR_BLOCK_LENGTH, S>, P1, P2),
    P1: 'static + FromParameters,
    P2: 'static + FromParameters,
    S: 'static + Default
{
    fn get_parameter_descriptors() -> [ParameterDescriptor; 2] {
        [P1::DESCRIPTOR, P2::DESCRIPTOR]
    }

    fn into_instrument_processor_implementation(self, sample_rate: f32) -> impl InstrumentProcessorImplementation {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
}