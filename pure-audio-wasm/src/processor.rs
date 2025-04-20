use std::marker::PhantomData;
use pure_audio::{AutomationRate, Event, IntoProcessor, OutEvent, ParameterDescriptor, Processor};
use wasm_bindgen::prelude::*;
use crate::PROCESSOR_BLOCK_LENGTH;

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

    pub fn get_parameters_per_sample_ptr(&mut self) -> usize {
        self.implementation.get_parameters_per_sample_ptr()
    }

    pub fn process(&mut self) {
        self.implementation.process();
    }

    pub fn note_on(&mut self, key: u8, velocity: u8) {
        self.implementation.note_on(key, velocity);
    }

    pub fn note_off(&mut self, key: u8, velocity: u8) {
        self.implementation.note_off(key, velocity);
    }

    pub fn indicate_params_changed(&mut self) {
        self.implementation.indicate_params_changed();
    }
}

pub trait WasmProcessorImplementation: 'static {
    fn get_inputs_ptr(&mut self) -> usize;
    fn get_outputs_ptr(&self) -> usize;
    fn get_parameters_ptr(&mut self) -> usize;
    fn get_parameters_per_sample_ptr(&mut self) -> usize;
    fn process(&mut self);
    fn note_on(&mut self, key: u8, velocity: u8);
    fn note_off(&mut self, key: u8, velocity: u8);
    fn indicate_params_changed(&mut self);
}

struct WasmProcessorWrapper<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params> {
    processor: P,
    events: Vec<Event>,
    out_events: Vec<OutEvent>,
    inputs: [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_INPUTS],
    outputs: [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS],
    parameters: [u32; NUM_PARAMS],
    parameters_per_sample: [Option<[u32; PROCESSOR_BLOCK_LENGTH]>; NUM_PARAMS],
    marker: PhantomData<Params>
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, Params, const NUM_PARAMS: usize> WasmProcessorWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
{
    fn new(processor: P, param_descriptors: &[(ParameterDescriptor, AutomationRate); NUM_PARAMS]) -> Self {
        Self {
            processor,
            events: vec![],
            out_events: vec![],
            inputs: [[[0.0; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_INPUTS],
            outputs: [[[0.0; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS],
            parameters: [0; NUM_PARAMS],
            parameters_per_sample: param_descriptors.map(|(.., automation_rate)| {
                match automation_rate {
                    AutomationRate::A => Some([0; PROCESSOR_BLOCK_LENGTH]),
                    AutomationRate::K => None
                }
            }),
            marker: PhantomData
        }
    }
}

pub trait IntoWasmProcessor<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> {
    const PARAM_DESCRIPTORS: [(ParameterDescriptor, AutomationRate); NUM_PARAMS];
    fn into_wasm_processor(self, sample_rate: f32) -> WasmProcessor;
}

pub trait IntoWasmProcessorImplementation<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> {
    const PARAM_DESCRIPTORS: [(ParameterDescriptor, AutomationRate); NUM_PARAMS];
    fn into_wasm_processor_implementation(self, sample_rate: f32) -> impl WasmProcessorImplementation;
}

impl<I, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> IntoWasmProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> for I
where
    I: IntoWasmProcessorImplementation<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    const PARAM_DESCRIPTORS: [(ParameterDescriptor, AutomationRate); NUM_PARAMS] = I::PARAM_DESCRIPTORS;

    fn into_wasm_processor(self, sample_rate: f32) -> WasmProcessor {
        WasmProcessor::new(Box::new(self.into_wasm_processor_implementation(sample_rate)))
    }
}

impl<P, Params, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize> WasmProcessorImplementation for WasmProcessorWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>
where
    P: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>,
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

    fn get_parameters_per_sample_ptr(&mut self) -> usize {
        self.parameters_per_sample.as_ptr() as *const _ as usize
    }

    fn process(&mut self) {
        self.out_events.clear();
        // clear outputs
        self.outputs = [[[0.0; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS];

        // map [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_INPUTS] -> [[&[f32]; NUM_CHANNELS]; NUM_INPUTS]
        let inputs = 
            self
                .inputs
                .each_ref()
                .map(|input| 
                    input
                        .each_ref()
                        .map(|channel| channel.as_ref())
                );

        // map [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS] -> [[&[f32]; NUM_CHANNELS]; NUM_OUTPUTS]
        let outputs =
                self
                    .outputs
                    .each_mut()
                    .map(|output|
                        output
                            .each_mut()
                            .map(|channel| channel.as_mut())
                    );
        
        let parameters_per_sample = self.parameters_per_sample.each_ref().map(|p| p.as_ref().map(|p| p.as_slice()));
        self.processor.process(inputs, outputs, &self.parameters, &parameters_per_sample, &self.events, &mut self.out_events);
        self.events.clear();
    }

    fn note_on(&mut self, key: u8, velocity: u8) {
        // currently not using port_index, channel and note_id from wasm
        self.events.push(Event::NoteOn { key, velocity, port_index: 0, channel: 0, note_id: 0 });
    }

    fn note_off(&mut self, key: u8, velocity: u8) {
        // currently not using port_id, channel and note_id from wasm
        self.events.push(Event::NoteOff { key, velocity, port_index: 0, channel: 0, note_id: 0 });
    }

    fn indicate_params_changed(&mut self) {
        self.events.push(Event::ParamsChanged);
    }
}

impl<F, A, Params, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, S> IntoWasmProcessorImplementation<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> for F
where 
    F: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
    Params: 'static,
    S: 'static + Default
{    
    const PARAM_DESCRIPTORS: [(ParameterDescriptor, AutomationRate); NUM_PARAMS] = F::PARAM_DESCRIPTORS;

    fn into_wasm_processor_implementation(self, sample_rate: f32) -> impl WasmProcessorImplementation {
        let mut processor = self.into_processor();
        processor.activate(sample_rate, PROCESSOR_BLOCK_LENGTH, PROCESSOR_BLOCK_LENGTH);
        WasmProcessorWrapper::new(processor, &F::PARAM_DESCRIPTORS)
    }
}