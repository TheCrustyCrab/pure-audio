use std::marker::PhantomData;
use pure_audio::{AutomationRate, Event, IntoProcessor, OutEvents, ParameterDescriptor, Processor};
use wasm_bindgen::prelude::*;
use crate::{event::WasmOutEventDispatcher, PROCESSOR_BLOCK_LENGTH};

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

trait WasmProcessorImplementation: 'static {
    fn get_inputs_ptr(&mut self) -> usize;
    fn get_outputs_ptr(&self) -> usize;
    fn get_parameters_ptr(&mut self) -> usize;
    fn get_parameters_per_sample_ptr(&mut self) -> usize;
    fn process(&mut self);
    fn note_on(&mut self, key: u8, velocity: u8);
    fn note_off(&mut self, key: u8, velocity: u8);
    fn indicate_params_changed(&mut self);
}

struct WasmProcessorWrapper<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> 
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
    A: 'static,
    Params: 'static,
    S: 'static + Default
{
    processor: P::Out,
    events: Vec<Event>,
    out_event_dispatcher: WasmOutEventDispatcher,
    inputs: [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_INPUTS],
    outputs: [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS],
    parameters: [u32; NUM_PARAMS],
    parameters_per_sample: [Option<[u32; PROCESSOR_BLOCK_LENGTH]>; NUM_PARAMS],
    marker: PhantomData<(A, Params, S)>
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> WasmProcessorWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
    A: 'static,
    Params: 'static,
    S: 'static + Default
{
    fn new(processor: P::Out, param_descriptors: &[(ParameterDescriptor, AutomationRate); NUM_PARAMS], output_event_callback: js_sys::Function) -> Self {
        Self {
            processor,
            events: vec![],
            out_event_dispatcher: WasmOutEventDispatcher::new(output_event_callback),
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

pub(crate) trait IntoWasmProcessor<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> {
    fn into_wasm_processor(self, sample_rate: f32, output_event_callback: js_sys::Function) -> WasmProcessor;
}

trait IntoWasmProcessorImplementation<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> {
    fn into_wasm_processor_implementation(self, sample_rate: f32, output_event_callback: js_sys::Function) -> impl WasmProcessorImplementation;
}

impl<I, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> IntoWasmProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> for I
where
    I: IntoWasmProcessorImplementation<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{

    fn into_wasm_processor(self, sample_rate: f32, output_event_callback: js_sys::Function) -> WasmProcessor {
        WasmProcessor::new(Box::new(self.into_wasm_processor_implementation(sample_rate, output_event_callback)))
    }
}

impl<P, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S> WasmProcessorImplementation for WasmProcessorWrapper<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
    A: 'static,
    Params: 'static,
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

    fn get_parameters_per_sample_ptr(&mut self) -> usize {
        self.parameters_per_sample.as_ptr() as *const _ as usize
    }

    fn process(&mut self) {
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

        // map [[[f32; PROCESSOR_BLOCK_LENGTH]; NUM_CHANNELS]; NUM_OUTPUTS] -> [[&mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS]
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
        let out_events = OutEvents::new(&self.out_event_dispatcher);
        self.processor.process(inputs, outputs, &self.parameters, &parameters_per_sample, &self.events, out_events);
        
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

impl<P, A, Params, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, S> IntoWasmProcessorImplementation<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> for P
where 
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
    A: 'static,
    Params: 'static,
    S: 'static + Default
{
    fn into_wasm_processor_implementation(self, sample_rate: f32, output_event_callback: js_sys::Function) -> impl WasmProcessorImplementation {
        let mut processor = self.into_processor();
        processor.activate(sample_rate, PROCESSOR_BLOCK_LENGTH, PROCESSOR_BLOCK_LENGTH);
        WasmProcessorWrapper::<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>::new(processor, &P::PARAM_DESCRIPTORS, output_event_callback)
    }
}