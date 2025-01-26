use std::fmt::{Display, Write};

pub trait FromParameters {
    const DESCRIPTOR: ParameterDescriptor;
    fn from_parameters(parameters: &[f32], index: usize) -> Self;
    fn text_to_value(text: &str) -> Option<f64>;
    fn value_to_text(value: f64, writer: &mut impl Write) -> bool;
}

#[derive(Copy, Clone)]
pub enum ParameterAutomationRate {
    A,
    K
}

impl Display for ParameterAutomationRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterAutomationRate::A => write!(f, "{}", "a-rate"),
            ParameterAutomationRate::K => write!(f, "{}", "k-rate"),
        }
    }
}

#[derive(Copy, Clone)]
pub struct ParameterDescriptor {
    pub name: &'static str,
    pub default_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub automation_rate: ParameterAutomationRate
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

impl<P: Parameter> FromParameters for P {
    const DESCRIPTOR: ParameterDescriptor = P::DESCRIPTOR;

    #[inline]
    fn from_parameters(parameters: &[f32], index: usize) -> Self {
        P::from_parameter(parameters[index])
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