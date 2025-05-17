use std::marker::PhantomData;
use pure_audio::IntoProcessor;

pub(crate) struct ParameterConverter<
    P,
    const NUM_INPUTS: usize,
    const NUM_OUTPUTS: usize,
    const NUM_CHANNELS: usize,
    const NUM_PARAMS: usize,
    A,
    Params,
    S,
> {
    text_buffer: String,
    marker: PhantomData<(P, A, Params, S)>,
}
impl<
        P,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const NUM_PARAMS: usize,
        A,
        Params,
        S,
    > ParameterConverter<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    pub(crate) fn new() -> Self {
        Self {
            text_buffer: String::new(),
            marker: PhantomData
        }
    }
}

pub(crate) trait ParameterConverterImplementation {
    fn value_to_text(&mut self, index: usize, value: f64) -> &str;
}

impl<
        P,
        const NUM_INPUTS: usize,
        const NUM_OUTPUTS: usize,
        const NUM_CHANNELS: usize,
        const NUM_PARAMS: usize,
        A,
        Params,
        S,
    > ParameterConverterImplementation
    for ParameterConverter<P, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
where
    P: 'static + IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>,
    A: 'static,
    Params: 'static,
    S: 'static + Default,
{
    fn value_to_text(&mut self, index: usize, value: f64) -> &str {
        self.text_buffer.clear();
        let _ = P::parameter_value_to_text(index, value, &mut self.text_buffer);
        &self.text_buffer
    }
}

pub(crate) struct WasmParameterConverter {
    implementation: Box<dyn ParameterConverterImplementation>,
}

impl WasmParameterConverter {
    pub(crate) fn new(implementation: Box<dyn ParameterConverterImplementation>) -> usize {        
        Box::into_raw(Box::new(Self { implementation })) as usize
    }

    pub(crate) fn value_to_text(&mut self, index: usize, value: f64) -> &str {
        self.implementation.value_to_text(index, value)
    }

    pub(crate) fn from_raw_ptr<'a>(ptr: usize) -> &'a mut Self {
        unsafe { Box::leak(Box::from_raw(ptr as *mut _)) }
    }

    pub(crate) fn destroy(ptr: usize) {
        unsafe { let _ = Box::from_raw(ptr as *mut Self); }
    }
}