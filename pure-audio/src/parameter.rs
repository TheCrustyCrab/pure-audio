use std::{fmt::{Display, Write}, marker::PhantomData};

#[derive(Copy, Clone)]
pub enum AutomationRate {
    A,
    K
}

impl Display for AutomationRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutomationRate::A => write!(f, "{}", "a-rate"),
            AutomationRate::K => write!(f, "{}", "k-rate"),
        }
    }
}

#[derive(Copy, Clone)]
pub enum ParameterKind {
    Bool,
    Enum(&'static [&'static str]),
    F32,
    I32,
    U32
}

#[derive(Copy, Clone)]
pub enum ParameterDescriptor {
    Local(LocalParameterDescriptor),
    Host
}

impl ParameterDescriptor {
    pub fn default_value(&self) -> f32 {
        match self {
            ParameterDescriptor::Local(local_parameter_descriptor) => local_parameter_descriptor.default_value,
            ParameterDescriptor::Host => 0.0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct LocalParameterDescriptor {
    pub name: &'static str,
    pub default_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub kind: ParameterKind
}

pub trait Parameter {
    const DESCRIPTOR: LocalParameterDescriptor;
    fn from_bits(value: u32) -> Self;
    fn f64_to_bits(d: f64) -> u32;
    fn bits_to_f64(value: u32) -> f64;
    fn text_to_value(text: &str) -> Option<f64> {
        text.parse::<f64>().ok()
    }
    fn value_to_text(value: f64, writer: &mut impl Write) -> bool {
        write!(writer, "{value}").is_ok()
    }
}

pub struct SamplePrecise<'a, P> { 
    values: &'a [u32],
    marker: PhantomData<P>
}

impl<P: Parameter> SamplePrecise<'_, P> {
    #[inline]
    pub fn values(&self) -> &[P] {
        unsafe { std::slice::from_raw_parts(self.values.as_ptr() as *const P, self.values.len()) }
    }
}

pub trait FromParameterBits {
    const DESCRIPTOR: LocalParameterDescriptor;
    const AUTOMATION_RATE: AutomationRate;
    type Out<'a>: FromParameterBits;
    fn from_parameter_bits<'a>(single: &'a [u32], per_sample: &'a [Option<&[u32]>], index: usize) -> Self::Out<'a>;
    fn f64_to_bits(d: f64) -> u32;
    fn bits_to_f64(value: u32) -> f64;
    fn text_to_value(text: &str) -> Option<f64>;
    fn value_to_text(value: f64, writer: &mut impl Write) -> bool;
}

// todo: disallow SamplePrecise for HostParameters
impl<P: Parameter> FromParameterBits for SamplePrecise<'_, P> {
    const DESCRIPTOR: LocalParameterDescriptor = P::DESCRIPTOR;
    const AUTOMATION_RATE: AutomationRate = AutomationRate::A;

    type Out<'a> = SamplePrecise<'a, P>;

    #[inline]
    fn from_parameter_bits<'a>(_single: &'a [u32], per_sample: &'a [Option<&[u32]>], index: usize) -> Self::Out<'a> {
        let values = per_sample[index].unwrap();
        SamplePrecise { values, marker: PhantomData }
    }
    
    #[inline]
    fn f64_to_bits(d: f64) -> u32 {
        P::f64_to_bits(d)
    }

    #[inline]
    fn bits_to_f64(value: u32) -> f64 {
        P::bits_to_f64(value)
    }

    #[inline]
    fn text_to_value(text: &str) -> Option<f64> {
        P::text_to_value(text)
    }

    #[inline]
    fn value_to_text(value: f64, writer: &mut impl Write) -> bool {
        P::value_to_text(value, writer)
    }
}

impl<P: Parameter> FromParameterBits for P {
    const DESCRIPTOR: LocalParameterDescriptor = P::DESCRIPTOR;
    const AUTOMATION_RATE: AutomationRate = AutomationRate::K;

    type Out<'a> = P;

    #[inline]
    fn from_parameter_bits<'a>(single: &'a [u32], _per_sample: &'a [Option<&[u32]>], index: usize) -> Self::Out<'a> {
        let value = single[index];
        P::from_bits(value)
    }
    
    #[inline]
    fn f64_to_bits(d: f64) -> u32 {
        P::f64_to_bits(d)
    }

    #[inline]
    fn bits_to_f64(value: u32) -> f64 {
        P::bits_to_f64(value)
    }

    #[inline]
    fn text_to_value(text: &str) -> Option<f64> {
        P::text_to_value(text)
    }

    #[inline]
    fn value_to_text(value: f64, writer: &mut impl Write) -> bool {
        P::value_to_text(value, writer)
    }
}

pub trait FromParameterContext {
    const DESCRIPTOR: ParameterDescriptor;
    const AUTOMATION_RATE: AutomationRate;
    type Out<'a>: FromParameterContext;
    fn from_parameter_context<'a>(context: ParameterContext<'a>, index: usize) -> Self::Out<'a>;
    fn f64_to_bits(d: f64) -> u32;
    fn bits_to_f64(value: u32) -> f64;
    fn text_to_value(text: &str) -> Option<f64>;
    fn value_to_text(value: f64, writer: &mut impl Write) -> bool;
}

#[derive(Clone, Copy)]
pub struct ParameterContext<'a> {
    parameter_single_values: &'a [u32],
    parameter_per_sample_values: &'a [Option<&'a [u32]>],
    host_parameters: &'a HostParameters
}

impl<'a> ParameterContext<'a> {
    #[inline]
    pub fn new(parameter_single_values: &'a [u32], parameter_per_sample_values: &'a [Option<&'a [u32]>], host_parameters: &'a HostParameters) -> Self {
        Self {
            parameter_single_values,
            parameter_per_sample_values,
            host_parameters
        }
    }
}

#[derive(Clone, Copy)]
pub struct HostParameters {
    pub tempo: f32
    // todo: song position etc
}

impl HostParameters {
    #[inline]
    pub fn new(tempo: f32) -> Self {
        Self { tempo }
    }
}

pub struct Tempo(f32);

impl FromParameterContext for Tempo {
    const DESCRIPTOR: ParameterDescriptor = ParameterDescriptor::Host;

    const AUTOMATION_RATE: AutomationRate = AutomationRate::K;

    type Out<'a> = Tempo;

    #[inline]
    fn from_parameter_context<'a>(context: ParameterContext<'a>, _index: usize) -> Self::Out<'a> {
        Tempo(context.host_parameters.tempo)
    }

    // values don't matter but called during initialisation
    fn f64_to_bits(_d: f64) -> u32 {
        0
    }

    fn bits_to_f64(_value: u32) -> f64 {
        0.0
    }

    fn text_to_value(_text: &str) -> Option<f64> {
        None
    }

    fn value_to_text(_value: f64, _writer: &mut impl Write) -> bool {
        false
    }
}

impl<F: FromParameterBits> FromParameterContext for F {
    const DESCRIPTOR: ParameterDescriptor = ParameterDescriptor::Local(F::DESCRIPTOR);
    const AUTOMATION_RATE: AutomationRate = F::AUTOMATION_RATE;

    type Out<'a> = F::Out<'a>;

    #[inline]
    fn from_parameter_context<'a>(context: ParameterContext<'a>, index: usize) -> Self::Out<'a> {
        F::from_parameter_bits(context.parameter_single_values, context.parameter_per_sample_values, index)
    }

    #[inline]
    fn f64_to_bits(d: f64) -> u32 {
        F::f64_to_bits(d)
    }

    #[inline]
    fn bits_to_f64(value: u32) -> f64 {
        F::bits_to_f64(value)
    }

    #[inline]
    fn text_to_value(text: &str) -> Option<f64> {
        F::text_to_value(text)
    }

    #[inline]
    fn value_to_text(value: f64, writer: &mut impl Write) -> bool {
        F::value_to_text(value, writer)
    }
}