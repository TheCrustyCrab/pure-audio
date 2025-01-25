use crate::{
    event::Event, AudioData, FromParameters, InputBuffer, OutputBuffer, ParameterDescriptor
};
use pure_audio_proc_macro::{for_params, impl_processor};
use std::{fmt::Write, marker::PhantomData};

pub trait Processor<
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    Params,
>
{
    // currently passing inputs by reference 
    // and outputs (the container struct with size = NUM_OUTPUTS * NUM_CHANNELS * 2 * word size) with ownership which might not be ideal for performance
    // reason: passing outputs by mutable reference causes an inconvenient &mut &mut f32 in the process functions
    fn process<'a>(
        &'a mut self,
        inputs: &'a [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
        parameters: &'a [f32; NUM_PARAMS],
        events: &'a [Event]
    ) {
    }

    fn set_sample_rate(&mut self, sample_rate: f32);
}

pub struct ProcessorWrapper<
    F,
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    Params,
    S,
> {
    f: F,
    sample_rate: f32,
    state: S,
    marker: PhantomData<Params>,
}

impl<
        F,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const NUM_PARAMS: usize,
        Params,
        S,
    >
    ProcessorWrapper<
        F,
        NUM_INPUTS,
        NUM_OUTPUTS,
        NUM_CHANNELS,
        NUM_PARAMS,
        Params,
        S,
    >
{
    fn new(f: F, sample_rate: f32, state: S) -> Self {
        Self {
            f,
            sample_rate,
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
    Params,
    S,
>
{
    type Out: 'static + Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params>;
    const PARAM_DESCRIPTORS: [ParameterDescriptor; NUM_PARAMS];
    fn into_processor(
        self,
        sample_rate: f32,
    ) -> Self::Out;
    fn parameter_text_to_value(index: usize, text: &str) -> Option<f64>;
    fn parameter_value_to_text<W: Write>(index: usize, value: f64, writer: &mut W) -> bool;
}

// Support process functions with up to 16 parameters
for_params!(impl_processor, 16);
