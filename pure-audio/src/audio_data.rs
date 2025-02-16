use crate::event::Event;

pub trait FromRawAudioData<
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    S,
>
{
    type Out<'a>: FromRawAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>
    where
        S: 'a;
    fn from_raw_audio_data<'a>(
        inputs: [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
        events: &'a [Event],
        sample_rate: f32,
        state: &'a mut S,
    ) -> Self::Out<'a>;
}

pub struct AudioData<
    'a,
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    S = (),
> {
    pub inputs: [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
    pub outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
    pub events: &'a [Event],
    pub sample_rate: f32,
    pub state: &'a mut S,
}

impl<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, S>
    FromRawAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>
    for AudioData<'_, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>
{
    type Out<'a> = AudioData<'a, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>
    where 
        S: 'a;

    #[inline]
    fn from_raw_audio_data<'a>(
        inputs: [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
        outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
        events: &'a [Event],
        sample_rate: f32,
        state: &'a mut S,
    ) -> Self::Out<'a> {
        AudioData {
            inputs,
            outputs,
            events,
            sample_rate,
            state
        }
    }
}

pub struct MonoEffectData<'a, S = ()> {
    pub input: &'a [f32],
    pub output: &'a mut [f32],
    pub events: &'a [Event],
    pub sample_rate: f32,
    pub state: &'a mut S,
}

impl<S> FromRawAudioData<1, 1, 1, S> for MonoEffectData<'_, S> {
    type Out<'a> = MonoEffectData<'a, S>
    where
        S: 'a;

    #[inline]
    fn from_raw_audio_data<'a>(
        [[input]]: [[&'a [f32]; 1]; 1],
        [[output]]: [[&'a mut [f32]; 1]; 1],
        events: &'a [Event],
        sample_rate: f32,
        state: &'a mut S,
    ) -> Self::Out<'a> {
        MonoEffectData {
            input,
            output,
            events,
            sample_rate,
            state
        }
    }
}

pub struct StereoEffectData<'a, S = ()> {
    pub inputs: [&'a [f32]; 2],
    pub outputs: [&'a mut [f32]; 2],
    pub events: &'a [Event],
    pub sample_rate: f32,
    pub state: &'a mut S,
}

impl<S> FromRawAudioData<1, 1, 2, S> for StereoEffectData<'_, S> {
    type Out<'a> = StereoEffectData<'a, S>
    where
        S: 'a;

    #[inline]
    fn from_raw_audio_data<'a>(
        [inputs]: [[&'a [f32]; 2]; 1],
        [outputs]: [[&'a mut [f32]; 2]; 1],
        events: &'a [Event],
        sample_rate: f32,
        state: &'a mut S,
    ) -> Self::Out<'a> {
        StereoEffectData {
            inputs,
            outputs,
            events,
            sample_rate,
            state
        }
    }
}

pub struct MonoSynthData<'a, S = ()> {
    pub output: &'a mut [f32],
    pub events: &'a [Event],
    pub sample_rate: f32,
    pub state: &'a mut S,
}

impl<S> FromRawAudioData<0, 1, 1, S> for MonoSynthData<'_, S> {
    type Out<'a> = MonoSynthData<'a, S>
    where
        S: 'a;

    #[inline]
    fn from_raw_audio_data<'a>(
        _input: [[&'a [f32]; 1]; 0],
        [[output]]: [[&'a mut [f32]; 1]; 1],
        events: &'a [Event],
        sample_rate: f32,
        state: &'a mut S,
    ) -> Self::Out<'a> {
        MonoSynthData {
            output,
            events,
            sample_rate,
            state
        }
    }
}

pub struct StereoSynthData<'a, S = ()> {
    pub outputs: [&'a mut [f32]; 2],
    pub events: &'a [Event],
    pub sample_rate: f32,
    pub state: &'a mut S,
}

impl<S> FromRawAudioData<0, 1, 2, S> for StereoSynthData<'_, S> {
    type Out<'a> = StereoSynthData<'a, S>
    where
        S: 'a;

    #[inline]
    fn from_raw_audio_data<'a>(
        _inputs: [[&'a [f32]; 2]; 0],
        [outputs]: [[&'a mut [f32]; 2]; 1],
        events: &'a [Event],
        sample_rate: f32,
        state: &'a mut S,
    ) -> Self::Out<'a> {
        StereoSynthData {
            outputs,
            events,
            sample_rate,
            state
        }
    }
}