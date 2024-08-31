use std::ops::{Deref, DerefMut};

pub type Buffer<const SIZE: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize> =
    [[[f32; BLOCK_SIZE]; NUM_CHANNELS]; SIZE];

pub struct InputBuffer<'a, const SIZE: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize>(
    pub &'a Buffer<SIZE, NUM_CHANNELS, BLOCK_SIZE>,
);

impl<'a, const SIZE: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize> InputBuffer<'a, SIZE, NUM_CHANNELS, BLOCK_SIZE> {
    #[inline]
    pub fn new(data: &'a Buffer<SIZE, NUM_CHANNELS, BLOCK_SIZE>) -> Self {
        Self(data)
    }
}

impl<'a, const SIZE: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize> Deref
    for InputBuffer<'a, SIZE, NUM_CHANNELS, BLOCK_SIZE>
{
    type Target = Buffer<SIZE, NUM_CHANNELS, BLOCK_SIZE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct OutputBuffer<'a, const SIZE: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize>(
    pub &'a mut Buffer<SIZE, NUM_CHANNELS, BLOCK_SIZE>,
);

impl<'a, const SIZE: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize> OutputBuffer<'a, SIZE, NUM_CHANNELS, BLOCK_SIZE> {
    #[inline]
    pub fn new(data: &'a mut Buffer<SIZE, NUM_CHANNELS, BLOCK_SIZE>) -> Self {
        Self(data)
    }
}

impl<'a, const SIZE: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize> Deref
    for OutputBuffer<'a, SIZE, NUM_CHANNELS, BLOCK_SIZE>
{
    type Target = Buffer<SIZE, NUM_CHANNELS, BLOCK_SIZE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, const SIZE: usize, const NUM_CHANNELS: usize, const BLOCK_SIZE: usize> DerefMut
    for OutputBuffer<'a, SIZE, NUM_CHANNELS, BLOCK_SIZE>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
