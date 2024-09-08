use crate::{
    EffectAudioData, FromParameters, InputBuffer, InstrumentAudioData, OutputBuffer,
    ParameterDescriptor,
};
use std::marker::PhantomData;

pub trait Processor<
    const IS_INSTRUMENT: bool,
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
    ) {
    }
}

pub struct ProcessorWrapper<
    F,
    const IS_INSTRUMENT: bool,
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
        const IS_INSTRUMENT: bool,
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
        IS_INSTRUMENT,
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
    const IS_INSTRUMENT: bool,
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
        IS_INSTRUMENT,
        NUM_INPUTS,
        NUM_OUTPUTS,
        NUM_CHANNELS,
        BLOCK_SIZE,
        NUM_PARAMS,
        Params,
    >;
}

// effect with 1 parameter
impl<
        F,
        P1,
        const IS_INSTRUMENT: bool,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const BLOCK_SIZE: usize,
        S,
    > Processor<IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,)>
    for ProcessorWrapper<F, false, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,), S>
where
    F: 'static + FnMut(EffectAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default,
{
    #[inline]
    fn process(
        &mut self,
        inputs: &[[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: &mut [[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_OUTPUTS],
        parameters: &[f32; 1],
    ) {
        let p1 = P1::from_parameters(parameters, 0);
        let data = EffectAudioData {
            inputs: InputBuffer::new(inputs),
            outputs: OutputBuffer::new(outputs),
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
    > IntoProcessor<false, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,), S> for F
where
    F: 'static + FnMut(EffectAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default,
{
    fn get_parameter_descriptors() -> [ParameterDescriptor; 1] {
        [P1::DESCRIPTOR]
    }

    fn into_processor(
        self,
        sample_rate: f32,
    ) -> impl Processor<false, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,)> {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
}

// effect with 2 parameters
impl<
        F,
        P1,
        P2,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const BLOCK_SIZE: usize,
        S,
    > Processor<false, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 2, (P1, P2)>
    for ProcessorWrapper<
        F,
        false,
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
        + FnMut(EffectAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1, P2),
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
    ) {
        let p1 = P1::from_parameters(parameters, 0);
        let p2 = P2::from_parameters(parameters, 1);
        let data = EffectAudioData {
            inputs: InputBuffer::new(inputs),
            outputs: OutputBuffer::new(outputs),
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
    > IntoProcessor<false, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 2, (P1, P2), S> for F
where
    F: 'static
        + FnMut(EffectAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1, P2),
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
    ) -> impl Processor<false, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 2, (P1, P2)> {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
}

// instrument with 1 parameter
impl<F, P1, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize, S>
    Processor<true, 0, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,)>
    for ProcessorWrapper<F, true, 0, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,), S>
where
    F: 'static + FnMut(InstrumentAudioData<NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default,
{
    #[inline]
    fn process(
        &mut self,
        _inputs: &[[[f32; BLOCK_SIZE]; NUM_CHANNELS]; 0],
        outputs: &mut [[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_OUTPUTS],
        parameters: &[f32; 1],
    ) {
        let p1 = P1::from_parameters(parameters, 0);
        let data = InstrumentAudioData {
            outputs: OutputBuffer::new(outputs),
            sample_rate: self.sample_rate,
            state: &mut self.state,
        };
        (self.f)(data, p1);
    }
}

impl<F, P1, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize, S>
    IntoProcessor<true, 0, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,), S> for F
where
    F: 'static + FnMut(InstrumentAudioData<NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1),
    P1: 'static + FromParameters,
    S: 'static + Default,
{
    fn get_parameter_descriptors() -> [ParameterDescriptor; 1] {
        [P1::DESCRIPTOR]
    }

    fn into_processor(
        self,
        sample_rate: f32,
    ) -> impl Processor<true, 0, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 1, (P1,)> {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
}

// instrument with 2 parameters
impl<
        F,
        P1,
        P2,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const BLOCK_SIZE: usize,
        S,
    > Processor<true, 0, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 2, (P1, P2)>
    for ProcessorWrapper<F, true, 0, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 2, (P1, P2), S>
where
    F: 'static + FnMut(InstrumentAudioData<NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1, P2),
    P1: 'static + FromParameters,
    P2: 'static + FromParameters,
    S: 'static + Default,
{
    #[inline]
    fn process(
        &mut self,
        _inputs: &[[[f32; BLOCK_SIZE]; NUM_CHANNELS]; 0],
        outputs: &mut [[[f32; BLOCK_SIZE]; NUM_CHANNELS]; NUM_OUTPUTS],
        parameters: &[f32; 2],
    ) {
        let p1 = P1::from_parameters(parameters, 0);
        let p2 = P2::from_parameters(parameters, 1);
        let data = InstrumentAudioData {
            outputs: OutputBuffer::new(outputs),
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
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const BLOCK_SIZE: usize,
        S,
    > IntoProcessor<true, 0, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 2, (P1, P2), S> for F
where
    F: 'static + FnMut(InstrumentAudioData<NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, S>, P1, P2),
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
    ) -> impl Processor<true, 0, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE, 2, (P1, P2)> {
        ProcessorWrapper::new(self, sample_rate, S::default())
    }
}
