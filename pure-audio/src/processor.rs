use crate::{
    event::Event, AutomationRate, FromParameterContext, FromRawAudioData, OutEvents, ParameterContext, ParameterDescriptor
};
use pure_audio_proc_macro::{for_params, impl_processor};
use std::{fmt::Write, marker::PhantomData};

pub trait State: Default {
    fn activate(&mut self, sample_rate: f32, min_frame_count: usize, max_frame_count: usize);
}

impl State for () {
    fn activate(&mut self, _sample_rate: f32, _min_frame_count: usize, _max_frame_count: usize) {}
}

pub trait Processor<
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    Params,
>
{
    fn activate(&mut self, sample_rate: f32, min_frame_count: usize, max_frame_count: usize);

    fn process<'a>(
        &'a mut self,
        inputs: [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
        parameter_context: ParameterContext<'a>,
        events: &'a [Event],
        out_events: OutEvents<'a>
    );
}

pub struct ProcessorWrapper<
    F,
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    A,
    Params,
    S,
> {
    f: F,
    state: S,
    marker: PhantomData<(A, Params)>,
}

impl<
        F,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const NUM_PARAMS: usize,
        A,
        Params,
        S,
    > ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    fn new(f: F, state: S) -> Self {
        Self {
            f,
            state,
            marker: PhantomData,
        }
    }
}

pub trait IntoProcessor<
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    A,
    Params,
    S,
>
{
    type Out: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>;
    const HOST_PARAM_FLAGS: [bool; NUM_PARAMS];
    const PARAM_DESCRIPTORS: [(ParameterDescriptor, AutomationRate); NUM_PARAMS];
    const PARAMS_COUNT: usize;
    fn get_local_param_descriptor(index: usize) -> (usize, (ParameterDescriptor, AutomationRate)) {        
        Self::local_param_descriptors_iter()
            .nth(index)
            .unwrap()
    }
    fn local_param_descriptors_iter() -> impl Iterator<Item = (usize, (ParameterDescriptor, AutomationRate))> {
        Self::PARAM_DESCRIPTORS
            .into_iter()
            .enumerate()
            .zip(Self::HOST_PARAM_FLAGS)
            .filter(|(_, source_is_host)| !*source_is_host)
            .map(|((real_index, desc), ..)| (real_index, desc))
    }
    fn into_processor(self) -> Self::Out;
    fn parameter_f64_to_value(index: usize, d: f64) -> u32;
    fn parameter_value_to_f64(index: usize, value: u32) -> f64;
    fn parameter_text_to_value(index: usize, text: &str) -> Option<f64>;
    fn parameter_value_to_text<W: Write>(index: usize, value: f64, writer: &mut W) -> bool;
}

// Support process functions with up to 16 parameters
for_params!(impl_processor, 16);