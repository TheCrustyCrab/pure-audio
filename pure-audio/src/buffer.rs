use std::ops::Deref;

pub type Buffer<'a, const SIZE: usize, const NUM_CHANNELS: usize> =
    [[&'a [f32]; NUM_CHANNELS]; SIZE];

pub type BufferMut<'a, const SIZE: usize, const NUM_CHANNELS: usize> =
    [[&'a mut [f32]; NUM_CHANNELS]; SIZE];

pub struct InputBuffer<'a, const SIZE: usize, const NUM_CHANNELS: usize>(
    pub &'a Buffer<'a, SIZE, NUM_CHANNELS>,
);

impl<'a, const SIZE: usize, const NUM_CHANNELS: usize> InputBuffer<'a, SIZE, NUM_CHANNELS> {
    #[inline]
    pub fn new(data: &'a Buffer<SIZE, NUM_CHANNELS>) -> Self {
        Self(data)
    }
}

impl<'a, const SIZE: usize, const NUM_CHANNELS: usize> Deref
    for InputBuffer<'a, SIZE, NUM_CHANNELS>
{
    type Target = Buffer<'a, SIZE, NUM_CHANNELS>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct OutputBuffer<'a, const SIZE: usize, const NUM_CHANNELS: usize>(
    pub BufferMut<'a, SIZE, NUM_CHANNELS>,
);

impl<'a, const SIZE: usize, const NUM_CHANNELS: usize> OutputBuffer<'a, SIZE, NUM_CHANNELS> {
    #[inline]
    pub fn new(data: BufferMut<'a, SIZE, NUM_CHANNELS>) -> Self {
        Self(data)
    }
}

impl<'a, const SIZE: usize, const NUM_CHANNELS: usize> Deref
    for OutputBuffer<'a, SIZE, NUM_CHANNELS>
{
    type Target = BufferMut<'a, SIZE, NUM_CHANNELS>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}