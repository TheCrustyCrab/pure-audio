use std::fmt::Display;

pub trait FromParameters {
    const DESCRIPTOR: ParameterDescriptor;
    fn from_parameters(parameters: &[f32], index: usize) -> Self;
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

pub trait ProcessorParameter {
    const DESCRIPTOR: ParameterDescriptor;
    fn from_parameter(value: f32) -> Self;
}

impl<P: ProcessorParameter> FromParameters for P {
    const DESCRIPTOR: ParameterDescriptor = P::DESCRIPTOR;

    fn from_parameters(parameters: &[f32], index: usize) -> Self {
        P::from_parameter(parameters[index])
    }    
}