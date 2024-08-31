use crate::buffer::{InputBuffer, OutputBuffer};

const DEFAULT_BLOCK_SIZE: usize = 128;

pub struct EffectAudioData<
    'a,
    const NUM_INPUTS: usize = 1,
    const NUM_OUTPUTS: usize = 1,
    const NUM_CHANNELS: usize = 1,
    const BLOCK_SIZE: usize = DEFAULT_BLOCK_SIZE,
    S = (),
> {
    pub inputs: InputBuffer<'a, NUM_INPUTS, NUM_CHANNELS, BLOCK_SIZE>,
    pub outputs: OutputBuffer<'a, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE>,
    pub sample_rate: f32,
    pub state: &'a mut S,
}

pub struct InstrumentAudioData<
    'a,
    const NUM_OUTPUTS: usize = 1,
    const NUM_CHANNELS: usize = 1,
    const BLOCK_SIZE: usize = DEFAULT_BLOCK_SIZE,
    S = (),
> {
    pub outputs: OutputBuffer<'a, NUM_OUTPUTS, NUM_CHANNELS, BLOCK_SIZE>,
    pub sample_rate: f32,
    pub state: &'a mut S,
}
