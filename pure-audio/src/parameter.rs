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
pub struct ParameterDescriptor {
    pub name: &'static str,
    pub default_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub kind: ParameterKind
}

pub trait Parameter {
    const DESCRIPTOR: ParameterDescriptor;
    fn from_parameter(value: u32) -> Self;
    fn f64_to_value(d: f64) -> u32;
    fn value_to_f64(value: u32) -> f64;
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

pub trait FromParameterValues {
    const DESCRIPTOR: ParameterDescriptor;
    const AUTOMATION_RATE: AutomationRate;
    type Out<'a>: FromParameterValues;
    fn from_parameter_values<'a>(single: &'a [u32], per_sample: &'a [Option<&[u32]>], index: usize) -> Self::Out<'a>;
    fn f64_to_value(d: f64) -> u32;
    fn value_to_f64(value: u32) -> f64;
    fn text_to_value(text: &str) -> Option<f64>;
    fn value_to_text(value: f64, writer: &mut impl Write) -> bool;
}

// todo: disallow SamplePrecise for HostParameters
impl<P: Parameter> FromParameterValues for SamplePrecise<'_, P> {
    const DESCRIPTOR: ParameterDescriptor = P::DESCRIPTOR;
    const AUTOMATION_RATE: AutomationRate = AutomationRate::A;

    type Out<'a> = SamplePrecise<'a, P>;

    #[inline]
    fn from_parameter_values<'a>(_single: &'a [u32], per_sample: &'a [Option<&[u32]>], index: usize) -> Self::Out<'a> {
        let values = per_sample[index].unwrap();
        SamplePrecise { values, marker: PhantomData }
    }
    
    #[inline]
    fn f64_to_value(d: f64) -> u32 {
        P::f64_to_value(d)
    }

    #[inline]
    fn value_to_f64(value: u32) -> f64 {
        P::value_to_f64(value)
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

impl<P: Parameter> FromParameterValues for P {
    const DESCRIPTOR: ParameterDescriptor = P::DESCRIPTOR;
    const AUTOMATION_RATE: AutomationRate = AutomationRate::K;

    type Out<'a> = P;

    #[inline]
    fn from_parameter_values<'a>(single: &'a [u32], _per_sample: &'a [Option<&[u32]>], index: usize) -> Self::Out<'a> {
        let value = single[index];
        P::from_parameter(value)
    }
    
    #[inline]
    fn f64_to_value(d: f64) -> u32 {
        P::f64_to_value(d)
    }

    #[inline]
    fn value_to_f64(value: u32) -> f64 {
        P::value_to_f64(value)
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
    const SOURCE_IS_HOST: bool;
    const DESCRIPTOR: ParameterDescriptor;
    const AUTOMATION_RATE: AutomationRate;
    type Out<'a>: FromParameterContext;
    fn from_parameter_context<'a>(context: ParameterContext<'a>, index: usize) -> Self::Out<'a>;
    fn f64_to_value(d: f64) -> u32;
    fn value_to_f64(value: u32) -> f64;
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
    const SOURCE_IS_HOST: bool = true;
    // won't be used for HostParameters
    const DESCRIPTOR: ParameterDescriptor = ParameterDescriptor {
        name: "n/a",
        default_value: 0.0,
        min_value: 0.0,
        max_value: 0.0,
        kind: ParameterKind::Bool,
    };

    const AUTOMATION_RATE: AutomationRate = AutomationRate::K;

    type Out<'a> = Tempo;

    #[inline]
    fn from_parameter_context<'a>(context: ParameterContext<'a>, _index: usize) -> Self::Out<'a> {
        Tempo(context.host_parameters.tempo)
    }

    // values don't matter but called during initialisation
    fn f64_to_value(d: f64) -> u32 {
        0
    }

    fn value_to_f64(value: u32) -> f64 {
        0.0
    }

    fn text_to_value(text: &str) -> Option<f64> {
        None
    }

    fn value_to_text(value: f64, writer: &mut impl Write) -> bool {
        false
    }
}

impl<F: FromParameterValues> FromParameterContext for F {
    const SOURCE_IS_HOST: bool = false;
    const DESCRIPTOR: ParameterDescriptor = F::DESCRIPTOR;
    const AUTOMATION_RATE: AutomationRate = F::AUTOMATION_RATE;

    type Out<'a> = F::Out<'a>;

    #[inline]
    fn from_parameter_context<'a>(context: ParameterContext<'a>, index: usize) -> Self::Out<'a> {
        F::from_parameter_values(context.parameter_single_values, context.parameter_per_sample_values, index)
    }

    #[inline]
    fn f64_to_value(d: f64) -> u32 {
        F::f64_to_value(d)
    }

    #[inline]
    fn value_to_f64(value: u32) -> f64 {
        F::value_to_f64(value)
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