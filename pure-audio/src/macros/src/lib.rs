use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, spanned::Spanned, token::Comma, Expr, Fields, FieldsUnnamed, Ident, Item, ItemStruct, LitFloat, LitInt, LitStr, Token};

struct ForParamsInput {
    macro_ident: Ident,
    max_params: usize
}

impl Parse for ForParamsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let max_params = input.parse::<LitInt>()?.base10_parse()?;

        Ok(ForParamsInput { 
            macro_ident,
            max_params
        })
    }
}

/// Calls the provided macro with each number of parameters in the range from 0 to the provided max_params (exclusive).
/// # Example
/// ```
/// macro_rules! my_macro {
///     ...
/// }
/// 
/// for_params!(my_macro, 3);
/// // my_macro!(0);
/// // my_macro!(1);
/// // my_macro!(2);
/// ```
#[proc_macro]
pub fn for_params(ts: TokenStream) -> TokenStream {
    let ForParamsInput { macro_ident, max_params } = parse_macro_input!(ts as ForParamsInput);
    let invocations =
        (0..max_params)
            .map(|num_params| quote! { #macro_ident!(#num_params); } );

    TokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}

struct ImplProcessorInput {
    num_params: usize
}

impl Parse for ImplProcessorInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let num_params = input.parse::<LitInt>()?.base10_parse()?;
        Ok(ImplProcessorInput { num_params })
    }
}

/// Generates an implementation of [`Processor`] for a process functions with a given number of parameters.
#[proc_macro]
pub fn impl_processor(ts: TokenStream) -> TokenStream {
    let ImplProcessorInput { num_params } = parse_macro_input!(ts as ImplProcessorInput);
    let (generic_idents, indices): (Vec<_>, Vec<_>) = 
        (0..num_params)
            .map(|index| (format_ident!("P{}", index + 1), syn::Index::from(index)))
            .unzip();
    
    let implementations = quote! {        
        impl<
                F,
                #(#generic_idents ,)*
                const NUM_INPUTS: usize,
                const NUM_OUTPUTS: usize,
                const NUM_CHANNELS: usize,
                S,
            > Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, #num_params, (#(#generic_idents,)*)>
            for ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, #num_params, (#(#generic_idents,)*), S>
        where
            F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>, #(#generic_idents),*) + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>, #(#generic_idents ::Out<'_>),*),
            #(
                #generic_idents: 'static + FromParameterValues,
            )*
            S: 'static + Default,
            {
                #[inline]
                fn process<'a>(
                    &'a mut self,
                    inputs: &'a [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
                    outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
                    parameter_single_values: &'a [f32; #num_params],
                    parameter_per_sample_values: &'a [Option<&'a [f32]>; #num_params],
                    events: &'a [Event]
                ) {
                    #(
                        let #generic_idents = #generic_idents::from_parameter_values(parameter_single_values, parameter_per_sample_values, #indices);
                    )*
                    let data = AudioData {
                        inputs: InputBuffer::new(inputs),
                        outputs: OutputBuffer::new(outputs),
                        events,
                        sample_rate: self.sample_rate,
                        state: &mut self.state,
                    };
                    (self.f)(data, #(#generic_idents),*);
                }

                #[inline]
                fn set_sample_rate(&mut self, sample_rate: f32) {
                    self.sample_rate = sample_rate;
                }
            }

        impl<
                F,
                #(#generic_idents ,)*
                const NUM_INPUTS: usize,
                const NUM_OUTPUTS: usize,
                const NUM_CHANNELS: usize,
                S,
            > IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, #num_params, (#(#generic_idents,)*), S> for F
        where
            F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>, #(#generic_idents,)*) + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>, #(#generic_idents ::Out<'_>),*),
            #(
                #generic_idents: 'static + FromParameterValues,
            )*
            S: 'static + Default,
            {
                type Out = ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, #num_params, (#(#generic_idents,)*), S>;
                const PARAM_DESCRIPTORS: [(ParameterDescriptor, AutomationRate); #num_params] = [
                    #(
                        (#generic_idents::DESCRIPTOR, #generic_idents::AUTOMATION_RATE)
                    ),*
                ];

                fn into_processor(
                    self,
                    sample_rate: f32,
                ) -> Self::Out {
                    ProcessorWrapper::new(self, sample_rate, S::default())
                }

                fn parameter_text_to_value(index: usize, text: &str) -> Option<f64> {
                    ([#(#generic_idents::text_to_value),*] as [fn(&str) -> Option<f64>; #num_params])[index](text)
                }

                fn parameter_value_to_text<W: Write>(index: usize, value: f64, writer: &mut W) -> bool {
                    ([#(#generic_idents::value_to_text),*] as [fn(f64, &mut W) -> bool; #num_params])[index](value, writer)
                }
            }
    };

    TokenStream::from(implementations)
}

enum ParameterAttr {
    Name(LitStr),
    DefaultValue(LitFloat),
    MinValue(LitFloat),
    MaxValue(LitFloat),
    TextToValue(Expr),
    ValueToText(Expr)
}

impl Parse for ParameterAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key_token = input.parse::<Ident>()?;
        let key = key_token.to_string();
        input.parse::<Token![=]>()?;
        match key.as_ref() {
            "name" => Ok(ParameterAttr::Name(input.parse()?)),
            "default" => Ok(ParameterAttr::DefaultValue(input.parse()?)),
            "min" => Ok(ParameterAttr::MinValue(input.parse()?)),
            "max" => Ok(ParameterAttr::MaxValue(input.parse()?)),
            "text_to_value" => Ok(ParameterAttr::TextToValue(input.parse()?)),
            "value_to_text" => Ok(ParameterAttr::ValueToText(input.parse()?)),
            _ => Err(syn::Error::new(key_token.span(), format!("attribute '{key}' is not supported")))
        }
    }
}

struct ParameterAttrs {
    name: Option<String>,
    default_value: Option<f32>,
    min_value: Option<f32>,
    max_value: Option<f32>,
    text_to_value: Option<Expr>,
    value_to_text: Option<Expr>
}

impl Parse for ParameterAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = syn::punctuated::Punctuated::<ParameterAttr, Comma>::parse_terminated(input)?;
        
        let mut name = None;
        let mut default_value = None;
        let mut min_value = None;
        let mut max_value = None;
        let mut text_to_value = None;
        let mut value_to_text = None;

        for attr in attrs {
            match attr {
                ParameterAttr::Name(lit_str) => name = Some(lit_str.value()),
                ParameterAttr::DefaultValue(lit_float) => default_value = Some(lit_float.base10_parse()?),
                ParameterAttr::MinValue(lit_float) => min_value = Some(lit_float.base10_parse()?),
                ParameterAttr::MaxValue(lit_float) => max_value = Some(lit_float.base10_parse()?),
                ParameterAttr::TextToValue(expr) => text_to_value = Some(expr),
                ParameterAttr::ValueToText(expr) => value_to_text = Some(expr)
            }
        }

        Ok(Self {
            name,
            default_value,
            min_value,
            max_value,
            text_to_value,
            value_to_text
        })
    }
}

/// Implements the [`Parameter`] trait for a parameter type.
/// The type must be a tuple struct with a single float field.
/// The following optional attributes can be provided:
/// - name: string literal (default: name of the type)
/// - default: float literal (default: 1.0)
/// - min: float literal (default: 0.0)
/// - max: float literal (default: 1.0)
/// - text_to_value: expression refering to a fn(&str) -> Option<f64> (default: built-in float parsing)
/// - value_to_text: expression refering to a fn(f64, &mut std::fmt::Write) -> bool (default: built-in float formatting)
#[proc_macro_attribute]
pub fn parameter(attr: TokenStream, input: TokenStream) -> TokenStream {
    match syn::parse::<syn::Item>(input) {
        Ok(item) => {
            if let Item::Struct(ref s) = item {
                let ItemStruct { ident, fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }), .. } = s else {
                    return TokenStream::from(syn::Error::new(item.span(), "type must be a tuple struct with a single field").into_compile_error());
                };

                if unnamed.len() != 1 {
                    return TokenStream::from(syn::Error::new(item.span(), "type must be a tuple struct with a single field").into_compile_error());
                }
                // todo: validate that the single unnamed field is f32/f64
                let struct_name = &ident;
                let ParameterAttrs { name, default_value, min_value, max_value, text_to_value, value_to_text }  = parse_macro_input!(attr as ParameterAttrs);

                let name = name.unwrap_or(struct_name.to_string());
                let default_value = default_value.unwrap_or(1.0);
                let min_value = min_value.unwrap_or(0.0);
                let max_value = max_value.unwrap_or(1.0);
                let text_to_value = if let Some(expr) = text_to_value {
                    quote! {
                        #[inline]
                        fn text_to_value(text: &str) -> Option<f64> {
                            #expr(text) 
                        }
                    }
                } else {
                    quote! { }
                };
                let value_to_text = if let Some(expr) = value_to_text {
                    quote! { 
                        #[inline]
                        fn value_to_text(value: f64, writer: &mut impl std::fmt::Write) -> bool {
                            #expr(value, writer)
                        } 
                    }
                } else {
                    quote! { }
                };
        
                let implementation = quote! {
                    #[derive(Copy, Clone)] 
                    #[pure_audio::pure_audio_proc_macro::parameter_arithmetic]
                    #s
        
                    impl pure_audio::Parameter for #struct_name {
                        const DESCRIPTOR: pure_audio::ParameterDescriptor = pure_audio::ParameterDescriptor {
                            name: #name,
                            default_value: #default_value,
                            min_value: #min_value,
                            max_value: #max_value
                        };
                        
                        #[inline]
                        fn from_parameter(value: f32) -> Self {
                            Self(value)
                        }
                        
                        #text_to_value

                        #value_to_text
                    }
                };

                TokenStream::from(implementation)
            } else {
                TokenStream::from(syn::Error::new(item.span(), "type not supported").into_compile_error())
            }
        },
        Err(err) => {
            TokenStream::from(err.into_compile_error())
        },
    }
}

/// Implements arithmetic operators for a parameter.
#[proc_macro_attribute]
pub fn parameter_arithmetic(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse::<syn::Item>(input).unwrap();
    if let syn::Item::Struct(s) = item {
        let name = &s.ident;
        let implementation = quote! {
            #s

            impl std::ops::Add<f32> for #name {
                type Output = f32;
            
                fn add(self, rhs: f32) -> Self::Output {
                    self.0 + rhs
                }
            }
            
            impl std::ops::Add<&f32> for #name {
                type Output = f32;
            
                fn add(self, rhs: &f32) -> Self::Output {
                    self.0 + rhs
                }
            }
            
            impl std::ops::Add<#name> for f32 {
                type Output = f32;
            
                fn add(self, rhs: #name) -> Self::Output {
                    self + rhs.0
                }
            }
            
            impl std::ops::Add<#name> for &f32 {
                type Output = f32;
            
                fn add(self, rhs: #name) -> Self::Output {
                    self + rhs.0
                }
            }
    
            impl std::ops::Div<f32> for #name {
                type Output = f32;
            
                fn div(self, rhs: f32) -> Self::Output {
                    self.0 / rhs
                }
            }
            
            impl std::ops::Div<&f32> for #name {
                type Output = f32;
            
                fn div(self, rhs: &f32) -> Self::Output {
                    self.0 / rhs
                }
            }
            
            impl std::ops::Div<#name> for f32 {
                type Output = f32;
            
                fn div(self, rhs: #name) -> Self::Output {
                    self / rhs.0
                }
            }
            
            impl std::ops::Div<#name> for &f32 {
                type Output = f32;
            
                fn div(self, rhs: #name) -> Self::Output {
                    self / rhs.0
                }
            }
    
            impl std::ops::Mul<f32> for #name {
                type Output = f32;
            
                fn mul(self, rhs: f32) -> Self::Output {
                    self.0 * rhs
                }
            }
            
            impl std::ops::Mul<&f32> for #name {
                type Output = f32;
            
                fn mul(self, rhs: &f32) -> Self::Output {
                    self.0 * rhs
                }
            }
            
            impl std::ops::Mul<#name> for f32 {
                type Output = f32;
            
                fn mul(self, rhs: #name) -> Self::Output {
                    self * rhs.0
                }
            }
            
            impl std::ops::Mul<#name> for &f32 {
                type Output = f32;
            
                fn mul(self, rhs: #name) -> Self::Output {
                    self * rhs.0
                }
            }
    
            impl std::ops::Sub<f32> for #name {
                type Output = f32;
            
                fn sub(self, rhs: f32) -> Self::Output {
                    self.0 - rhs
                }
            }
            
            impl std::ops::Sub<&f32> for #name {
                type Output = f32;
            
                fn sub(self, rhs: &f32) -> Self::Output {
                    self.0 - rhs
                }
            }
            
            impl std::ops::Sub<#name> for f32 {
                type Output = f32;
            
                fn sub(self, rhs: #name) -> Self::Output {
                    self - rhs.0
                }
            }
            
            impl std::ops::Sub<#name> for &f32 {
                type Output = f32;
            
                fn sub(self, rhs: #name) -> Self::Output {
                    self - rhs.0
                }
            }
        };
    
        TokenStream::from(implementation)
    } else {
        TokenStream::from(syn::Error::new(item.span(), "type not supported").into_compile_error())
    }
}