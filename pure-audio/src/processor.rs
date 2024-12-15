use crate::{
    event::Event, AudioData, FromParameters, InputBuffer, OutputBuffer, ParameterDescriptor
};
use std::marker::PhantomData;

pub trait Processor<
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const BLOCK_SIZE: usize,
    const NUM_PARAMS: usize,
    Params,
>
{
    fn process(
        &mut self,
        inputs: &[[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: &mut [[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_OUTPUTS],
        parameters: &[f32; NUM_PARAMS],
        events: &[Event]
    ) {
    }
}

pub struct ProcessorWrapper<
    F,
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const BLOCK_SIZE: usize,
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
        const BLOCK_SIZE: usize,
        const NUM_PARAMS: usize,
        Params,
        S,
    >
    ProcessorWrapper<
        F,
        NUM_INPUTS,
        NUM_OUTPUTS,
        NUM_CHANNELS,
        BLOCK_SIZE,
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
    const BLOCK_SIZE: usize,
    const NUM_PARAMS: usize,
    Params,
    S,
>
{
    fn get_parameter_descriptors() -> [ParameterDescriptor; NUM_PARAMS];
    fn into_processor(
        self,
        sample_rate: f32,
    ) -> impl Processor<
        NUM_INPUTS,
        NUM_OUTPUTS,
        NUM_CHANNELS,
        BLOCK_SIZE,
        NUM_PARAMS,
        Params,
    >;
}

// 0 parameters
impl<F, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize, S>
    Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 0, ()>
    for ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 0, (), S>
where 
    F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>),
    S: 'static + Default
{
    #[inline]
    fn process(
        &mut self,
        inputs: &[[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: &mut [[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_OUTPUTS],
        _parameters: &[f32; 0],
        events: &[Event]
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
}

impl<F, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize, S>
    IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 0, (), S> for F
where
    F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>),
    S: 'static + Default,
{
    fn get_parameter_descriptors() -> [ParameterDescriptor; 0] {
        []
    }

    fn into_processor(
        self,
        sample_rate: f32,
    ) -> impl Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 0, ()> {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
}

// 1 parameter
impl<
        F,
        P1,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const BLOCK_SIZE: usize,
        S,
    > Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,)>
    for ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,), S>
where
    F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default,
{
    #[inline]
    fn process(
        &mut self,
        inputs: &[[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: &mut [[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_OUTPUTS],
        parameters: &[f32; 1],
        events: &[Event]
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
}

impl<
        F,
        P1,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const BLOCK_SIZE: usize,
        S,
    > IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,), S> for F
where
    F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default,
{
    fn get_parameter_descriptors() -> [ParameterDescriptor; 1] {
        [P1::DESCRIPTOR]
    }

    fn into_processor(
        self,
        sample_rate: f32,
    ) -> impl Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,)> {
        ProcessorWrapper::new(self, sample_rate, S::default())
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
        const BLOCK_SIZE: usize,
        S,
    > Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 2, (P1, P2)>
    for ProcessorWrapper<
        F,
        NUM_INPUTS,
        NUM_OUTPUTS,
        NUM_CHANNELS,
        BLOCK_SIZE,
        2,
        (P1, P2),
        S,
    >
where
    F: 'static
        + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1, P2),
    P1: 'static + FromParameters,
    P2: 'static + FromParameters,
    S: 'static + Default,
{
    #[inline]
    fn process(
        &mut self,
        inputs: &[[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: &mut [[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_OUTPUTS],
        parameters: &[f32; 2],
        events: &[Event]
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
}

impl<
        F,
        P1,
        P2,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const BLOCK_SIZE: usize,
        S,
    > IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 2, (P1, P2), S> for F
where
    F: 'static
        + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1, P2),
    P1: 'static + FromParameters,
    P2: 'static + FromParameters,
    S: 'static + Default,
{
    fn get_parameter_descriptors() -> [ParameterDescriptor; 2] {
        [P1::DESCRIPTOR, P2::DESCRIPTOR]
    }

    fn into_processor(
        self,
        sample_rate: f32,
    ) -> impl Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 2, (P1, P2)> {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
}
