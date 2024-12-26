use crate::{buffer::{InputBuffer, OutputBuffer}, event::Event};

pub struct AudioData<
    'a,
    const NUM_INPUTS: usize = 1,
    const NUM_OUTPUTS: usize = 1,
    const NUM_CHANNELS: usize = 1,
    S = (),
> {
    pub inputs: InputBuffer<'a, NUM_INPUTS, NUM_CHANNELS>,
    pub outputs: OutputBuffer<'a, NUM_OUTPUTS, NUM_CHANNELS>,
    pub events: &'a [Event],
    pub sample_rate: f32,
    pub state: &'a mut S,
}
