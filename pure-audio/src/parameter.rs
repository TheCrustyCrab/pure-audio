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
pub struct ParameterDescriptor {
    pub name: &'static str,
    pub default_value: f32,
    pub min_value: f32,
    pub max_value: f32
}

pub trait Parameter {
    const DESCRIPTOR: ParameterDescriptor;
    fn from_parameter(value: f32) -> Self;
    fn text_to_value(text: &str) -> Option<f64> {
        text.parse::<f64>().ok()
    }
    fn value_to_text(value: f64, writer: &mut impl Write) -> bool {
        write!(writer, "{value}").is_ok()
    }
}

pub struct SamplePrecise<'a, P> { 
    pub values: &'a [f32],
    marker: PhantomData<P>
}

pub trait FromParameterValues {
    const DESCRIPTOR: ParameterDescriptor;
    const AUTOMATION_RATE: AutomationRate;
    type Out<'a>: FromParameterValues;
    fn from_parameter_values<'a>(single: &'a [f32], per_sample: &'a [Option<&[f32]>], index: usize) -> Self::Out<'a>;
    fn text_to_value(text: &str) -> Option<f64>;
    fn value_to_text(value: f64, writer: &mut impl Write) -> bool;
}

impl<P: Parameter> FromParameterValues for SamplePrecise<'_, P> {
    const DESCRIPTOR: ParameterDescriptor = P::DESCRIPTOR;
    const AUTOMATION_RATE: AutomationRate = AutomationRate::A;

    type Out<'a> = SamplePrecise<'a, P>;

    #[inline]
    fn from_parameter_values<'a>(_single: &'a [f32], per_sample: &'a [Option<&[f32]>], index: usize) -> Self::Out<'a> {
        let values = per_sample[index].unwrap();
        SamplePrecise { values, marker: PhantomData }
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
    fn from_parameter_values<'a>(single: &'a [f32], _per_sample: &'a [Option<&[f32]>], index: usize) -> Self::Out<'a> {
        let value = single[index];
        P::from_parameter(value)
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