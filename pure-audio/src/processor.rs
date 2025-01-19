use crate::{
    event::Event, AudioData, FromParameters, InputBuffer, OutputBuffer, ParameterDescriptor
};
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
    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS];
    const PARAM_DESCRIPTORS: [ParameterDescriptor; NUM_PARAMS];
    fn into_processor(
        self,
        sample_rate: f32,
    ) -> Self::Out;
    fn parameter_text_to_value(index: usize, text: &str) -> Option<f64>;
    fn parameter_value_to_text(index: usize, value: f64, writer: &mut impl Write) -> bool;
}

// 0 parameters
impl<F, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S>
    Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 0, ()>
    for ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 0, (), S>
where 
    F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>),
    S: 'static + Default
{
    #[inline]
    fn process<'a>(
        &'a mut self,
        inputs: &'a [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
        _parameters: &'a [f32; 0],
        events: &'a [Event]
    ) {
        let data = AudioData {
            events,
            inputs: InputBuffer::new(inputs),
            outputs: OutputBuffer::new(outputs),
            sample_rate: self.sample_rate,
            state: &mut self.state
        };
        (self.f)(data)
    }
    
    #[inline]
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

impl<F, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S>
    IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 0, (), S> for F
where
    F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>),
    S: 'static + Default,
{    
    type Out = ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 0, (), S>;
    const PARAM_DESCRIPTORS: [ParameterDescriptor; 0] = [];
    fn get_parameter_descriptors() -> [ParameterDescriptor; 0] {
        []
    }

    fn into_processor(
        self,
        sample_rate: f32,
    ) -> Self::Out {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
    
    fn parameter_text_to_value(index: usize, text: &str) -> Option<f64> {
        None
    }
    
    fn parameter_value_to_text(index: usize, value: f64, writer: &mut impl Write) -> bool {
        false
    }
}

// 1 parameter
impl<
        F,
        P1,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        S,
    > Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 1, (P1,)>
    for ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 1, (P1,), S>
where
    F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default,
{
    #[inline]
    fn process<'a>(
        &'a mut self,
        inputs: &'a [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
        parameters: &'a [f32; 1],
        events: &'a [Event]
    ) {
        let p1 = P1::from_parameters(parameters, 0);
        let data = AudioData {
            inputs: InputBuffer::new(inputs),
            outputs: OutputBuffer::new(outputs),
            events,
            sample_rate: self.sample_rate,
            state: &mut self.state,
        };
        (self.f)(data, p1);
    }
    
    #[inline]
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

impl<
        F,
        P1,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        S,
    > IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 1, (P1,), S> for F
where
    F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default,
{
    type Out = ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 1, (P1,), S>;
    const PARAM_DESCRIPTORS: [ParameterDescriptor; 1] = [P1::DESCRIPTOR];
    fn get_parameter_descriptors() -> [ParameterDescriptor; 1] {
        [P1::DESCRIPTOR]
    }

    fn into_processor(
        self,
        sample_rate: f32,
    ) -> Self::Out {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
    
    fn parameter_text_to_value(index: usize, text: &str) -> Option<f64> {
        P1::text_to_value(text)
    }
    
    fn parameter_value_to_text(index: usize, value: f64, writer: &mut impl Write) -> bool {
        P1::value_to_text(value, writer)
    }
}

// 2 parameters
impl<
        F,
        P1,
        P2,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        S,
    > Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 2, (P1, P2)>
    for ProcessorWrapper<
        F,
        NUM_INPUTS,
        NUM_OUTPUTS,
        NUM_CHANNELS,
        2,
        (P1, P2),
        S,
    >
where
    F: 'static
        + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>, P1, P2),
    P1: 'static + FromParameters,
    P2: 'static + FromParameters,
    S: 'static + Default,
{
    #[inline]
    fn process<'a>(
        &'a mut self,
        inputs: &'a [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
        parameters: &'a [f32; 2],
        events: &'a [Event]
    ) {
        let p1 = P1::from_parameters(parameters, 0);
        let p2 = P2::from_parameters(parameters, 1);
        let data = AudioData {
            inputs: InputBuffer::new(inputs),
            outputs: OutputBuffer::new(outputs),
            events,
            sample_rate: self.sample_rate,
            state: &mut self.state,
        };
        (self.f)(data, p1, p2);
    }
    
    #[inline]
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

impl<
        F,
        P1,
        P2,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        S,
    > IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 2, (P1, P2), S> for F
where
    F: 'static
        + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>, P1, P2),
    P1: 'static + FromParameters,
    P2: 'static + FromParameters,
    S: 'static + Default,
{
    type Out = ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, 2, (P1, P2), S>;
    const PARAM_DESCRIPTORS: [ParameterDescriptor; 2] = [P1::DESCRIPTOR, P2::DESCRIPTOR];
    fn get_parameter_descriptors() -> [ParameterDescriptor; 2] {
        [P1::DESCRIPTOR, P2::DESCRIPTOR]
    }

    fn into_processor(
        self,
        sample_rate: f32,
    ) -> Self::Out {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
    
    fn parameter_text_to_value(index: usize, text: &str) -> Option<f64> {
        [P1::text_to_value, P2::text_to_value][index](text)
    }
    
    fn parameter_value_to_text(index: usize, value: f64, writer: &mut impl Write) -> bool {
        [P1::value_to_text, P2::value_to_text][index](value, writer)        
    }
}
