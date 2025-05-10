use proc_macro::TokenStream;
use proc_macro2::{Punct, Spacing, Span, TokenTree};
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, spanned::Spanned, token::Comma, Expr, Fields, FieldsUnnamed, Ident, Item, ItemStruct, LitBool, LitFloat, LitInt, LitStr, Token, Type, TypePath};

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
/// use pure_audio_proc_macro::for_params;
/// macro_rules! my_macro {
///     ($e:expr) => {
///         $e
///     }
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
                A,
                #(#generic_idents ,)*
                const NUM_INPUTS: usize,
                const NUM_OUTPUTS: usize,
                const NUM_CHANNELS: usize,
                S,
            > Processor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, #num_params, (#(#generic_idents,)*)>
            for ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, #num_params, A, (#(#generic_idents,)*), S>
        where
            F: 'static + FnMut(A, #(#generic_idents),*) + FnMut(A::Out<'_>, #(#generic_idents ::Out<'_>),*),
            A: 'static + FromRawAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>,
            #(
                #generic_idents: 'static + FromParameterValues,
            )*
            S: 'static + State,
            {
                #[inline]
                fn activate(&mut self, sample_rate: f32, min_frame_count: usize, max_frame_count: usize) {
                    self.state.activate(sample_rate, min_frame_count, max_frame_count);
                }

                #[inline]
                fn process<'a>(
                    &'a mut self,
                    inputs: [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
                    outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
                    parameter_single_values: &'a [u32; #num_params],
                    parameter_per_sample_values: &'a [Option<&'a [u32]>; #num_params],
                    events: &'a [Event],
                    out_events: OutEvents<'a>
                ) {
                    #(
                        let #generic_idents = #generic_idents::from_parameter_values(parameter_single_values, parameter_per_sample_values, #indices);
                    )*
                    let data = A::from_raw_audio_data(inputs, outputs, events, out_events, &mut self.state);
                    (self.f)(data, #(#generic_idents),*);
                }
            }

        impl<
                F,
                A,
                #(#generic_idents ,)*
                const NUM_INPUTS: usize,
                const NUM_OUTPUTS: usize,
                const NUM_CHANNELS: usize,
                S,
            > IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, #num_params, A, (#(#generic_idents,)*), S> for F
        where
            F: 'static + FnMut(A, #(#generic_idents,)*) + FnMut(A::Out<'_>, #(#generic_idents ::Out<'_>),*),
            A: 'static + FromRawAudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>,
            #(
                #generic_idents: 'static + FromParameterValues,
            )*
            S: 'static + State,
            {
                type Out = ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, #num_params, A, (#(#generic_idents,)*), S>;
                const PARAM_DESCRIPTORS: [(ParameterDescriptor, AutomationRate); #num_params] = [
                    #(
                        (#generic_idents::DESCRIPTOR, #generic_idents::AUTOMATION_RATE)
                    ),*
                ];

                fn into_processor(
                    self
                ) -> Self::Out {
                    ProcessorWrapper::new(self, S::default())
                }

                fn parameter_f64_to_value(index: usize, d: f64) -> u32 {
                    ([#(#generic_idents::f64_to_value),*] as [fn(f64) -> u32; #num_params])[index](d)
                }

                fn parameter_value_to_f64(index: usize, value: u32) -> f64 {
                    ([#(#generic_idents::value_to_f64),*] as [fn(u32) -> f64; #num_params])[index](value)
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

enum NumericParameterAttr<const ONLY_INTEGERS: bool, const ONLY_UNSIGNED: bool> {
    LitFloat(LitFloat),
    LitInt(LitInt)
}

impl<const ONLY_INTEGERS: bool, const ONLY_UNSIGNED: bool> TryFrom<NumericParameterAttr<ONLY_INTEGERS, ONLY_UNSIGNED>> for f32 {
    type Error = syn::Error;

    fn try_from(value: NumericParameterAttr<ONLY_INTEGERS, ONLY_UNSIGNED>) -> Result<Self, Self::Error> {
        let (value, span): (f32, Span) = match value {
            NumericParameterAttr::LitFloat(lit_float) => (lit_float.base10_parse()?, lit_float.span()),
            NumericParameterAttr::LitInt(lit_int) => (lit_int.base10_parse()?, lit_int.span()),
        };
        
        if ONLY_INTEGERS && value.fract() != 0.0 {
            return Err(syn::Error::new(span, "values with fractions are not supported for i32 and u32"));
        }

        if ONLY_UNSIGNED && value < 0.0 {
            return Err(syn::Error::new(span, "negative values are not supported for u32"));
        }

        Ok(value)
    }
}

impl<const ONLY_INTEGERS: bool, const ONLY_UNSIGNED: bool> Parse for NumericParameterAttr<ONLY_INTEGERS, ONLY_UNSIGNED> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitFloat) {
            return Ok(Self::LitFloat(input.parse::<LitFloat>().unwrap()));
        }

        if let Ok(x) = input.parse::<LitInt>() {
            return Ok(Self::LitInt(x));
        }

        Err(syn::Error::new(input.span(), format!("expected a float or int literal")))
    }
}

enum ParameterAttr<const ONLY_INTEGERS: bool, const ONLY_UNSIGNED: bool> {
    Name(LitStr),
    DefaultValue(NumericParameterAttr<ONLY_INTEGERS, ONLY_UNSIGNED>),
    MinValue(NumericParameterAttr<ONLY_INTEGERS, ONLY_UNSIGNED>),
    MaxValue(NumericParameterAttr<ONLY_INTEGERS, ONLY_UNSIGNED>),
    TextToValue(Expr),
    ValueToText(Expr)
}

impl<const ONLY_INTEGERS: bool, const ONLY_UNSIGNED: bool> Parse for ParameterAttr<ONLY_INTEGERS, ONLY_UNSIGNED> {
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

struct ParameterAttrs<const ONLY_INTEGERS: bool, const ONLY_UNSIGNED: bool> {
    name: Option<String>,
    default_value: Option<f32>,
    min_value: Option<f32>,
    max_value: Option<f32>,
    text_to_value: Option<Expr>,
    value_to_text: Option<Expr>
}

impl<const ONLY_INTEGERS: bool, const ONLY_UNSIGNED: bool> Parse for ParameterAttrs<ONLY_INTEGERS, ONLY_UNSIGNED> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = syn::punctuated::Punctuated::<ParameterAttr<ONLY_INTEGERS, ONLY_UNSIGNED>, Comma>::parse_terminated(input)?;
        
        let mut name = None;
        let mut default_value = None;
        let mut min_value = None;
        let mut max_value = None;
        let mut text_to_value = None;
        let mut value_to_text = None;

        for attr in attrs {
            match attr {
                ParameterAttr::Name(lit_str) => name = Some(lit_str.value()),
                ParameterAttr::DefaultValue(numeric) => default_value = Some(numeric.try_into()?),
                ParameterAttr::MinValue(numeric) => min_value = Some(numeric.try_into()?),
                ParameterAttr::MaxValue(numeric) => max_value = Some(numeric.try_into()?),
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

enum BoolParameterAttr {
    Name(LitStr),
    DefaultValue(LitBool),
    TextToValue(Expr),
    ValueToText(Expr)
}

impl Parse for BoolParameterAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key_token = input.parse::<Ident>()?;
        let key = key_token.to_string();
        input.parse::<Token![=]>()?;
        match key.as_ref() {
            "name" => Ok(BoolParameterAttr::Name(input.parse()?)),
            "default" => Ok(BoolParameterAttr::DefaultValue(input.parse()?)),
            "text_to_value" => Ok(BoolParameterAttr::TextToValue(input.parse()?)),
            "value_to_text" => Ok(BoolParameterAttr::ValueToText(input.parse()?)),
            _ => Err(syn::Error::new(key_token.span(), format!("attribute '{key}' is not supported for bool types")))
        }
    }
}

struct BoolParameterAttrs {
    name: Option<String>,
    default_value: Option<f32>,
    text_to_value: Option<Expr>,
    value_to_text: Option<Expr>,
}

impl Parse for BoolParameterAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = syn::punctuated::Punctuated::<BoolParameterAttr, Comma>::parse_terminated(input)?;
        
        let mut name = None;
        let mut default_value = None;
        let mut text_to_value = None;
        let mut value_to_text = None;

        for attr in attrs {
            match attr {
                BoolParameterAttr::Name(lit_str) => name = Some(lit_str.value()),
                BoolParameterAttr::DefaultValue(lit_bool) => default_value = Some(if lit_bool.value { 1f32 } else { 0f32 }),
                BoolParameterAttr::TextToValue(expr) => text_to_value = Some(expr),
                BoolParameterAttr::ValueToText(expr) => value_to_text = Some(expr)
            }
        }

        Ok(Self {
            name,
            default_value,
            text_to_value,
            value_to_text
        })
    }
}

enum EnumParameterAttr {
    Name(LitStr),
    DefaultValue(LitStr),
    TextToValue(Expr),
    ValueToText(Expr)
}

impl Parse for EnumParameterAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key_token = input.parse::<Ident>()?;
        let key = key_token.to_string();
        input.parse::<Token![=]>()?;
        match key.as_ref() {
            "name" => Ok(EnumParameterAttr::Name(input.parse()?)),
            "default" => Ok(EnumParameterAttr::DefaultValue(input.parse()?)),
            "text_to_value" => Ok(EnumParameterAttr::TextToValue(input.parse()?)),
            "value_to_text" => Ok(EnumParameterAttr::ValueToText(input.parse()?)),
            _ => Err(syn::Error::new(key_token.span(), format!("attribute '{key}' is not supported for enum types")))
        }
    }
}

struct EnumParameterAttrs {
    name: Option<String>,
    default_value: Option<f32>,
    text_to_value: Option<Expr>,
    value_to_text: Option<Expr>
}

impl EnumParameterAttrs {
    fn from_input_and_variants(input: TokenStream, variants: &Vec<String>) -> syn::Result<Self> {
        let attrs = syn::parse::Parser::parse2(
            syn::punctuated::Punctuated::<EnumParameterAttr, Comma>::parse_terminated,
            input.into()
        )?;
        
        let mut name = None;
        let mut default_value = None;
        let mut text_to_value = None;
        let mut value_to_text = None;

        for attr in attrs {
            match attr {
                EnumParameterAttr::Name(lit_str) => name = Some(lit_str.value()),
                EnumParameterAttr::DefaultValue(lit_str) => {
                    let Some(value) = variants.iter().position(|variant| variant == &lit_str.value()) else {
                        return Err(syn::Error::new(lit_str.span(), "value is not a variant of the enum"));
                    };
                    default_value = Some(value as f32);
                },
                EnumParameterAttr::TextToValue(expr) => text_to_value = Some(expr),
                EnumParameterAttr::ValueToText(expr) => value_to_text = Some(expr)
            }
        }

        Ok(Self {
            name,
            default_value,
            text_to_value,
            value_to_text
        })
    }
}

enum SupportedNewType {
    Bool,
    F32,
    I32,
    U32
}

/// Implements the [`Parameter`] trait for a parameter type.
/// The type must be an enum or a tuple struct with a single bool, f32, i32 or u32 field.
/// The following optional attributes can be provided:
/// - name: string literal (default: name of the parameter)
/// - default: 
///     - boolean type: bool literal (default: false)
///     - enum type: string literal (default: first variant)
///     - numeric types: float literal (default: 1.0)
/// - min: float literal (default: 0.0, not for bools and enums)
/// - max: float literal (default: 1.0, not for bools and enums)
/// - text_to_value: expression refering to a [`fn(&str) -> Option<f64>`] (default: built-in float parsing)
/// - value_to_text: expression refering to a [`fn(f64, &mut impl std::fmt::Write) -> bool`] (default: built-in float formatting)
#[proc_macro_attribute]
pub fn parameter(attr: TokenStream, input: TokenStream) -> TokenStream {
    parameter_impl(attr, input).unwrap_or_else(|e| TokenStream::from(e.into_compile_error()))
}

// a separate function with error propagation
fn parameter_impl(attr: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
    const PARAMETER_TYPE_VALIDATION_MESSAGE: &'static str = "type must be a tuple struct with a single bool, f32, i32 or u32 field";
    let item = syn::parse::<syn::Item>(input)?;    
    match item {
        Item::Struct(ref s) => {
            let ItemStruct { ident, fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }), .. } = s else {
                return Err(syn::Error::new(item.span(), PARAMETER_TYPE_VALIDATION_MESSAGE));
            };

            if unnamed.len() != 1 {
                return Err(syn::Error::new(item.span(), PARAMETER_TYPE_VALIDATION_MESSAGE));
            }

            let new_type = if let Type::Path(TypePath { path, .. }) = &unnamed[0].ty {
                let ident = path.get_ident().map(|ident| ident.to_string());
                match ident.as_ref().map(String::as_str) {
                    Some("bool") => Some(SupportedNewType::Bool),
                    Some("f32") => Some(SupportedNewType::F32),
                    Some("i32") => Some(SupportedNewType::I32),
                    Some("u32") => Some(SupportedNewType::U32),
                    _ => None
                }
            } else {
                None
            };

            let Some(new_type) = new_type else {                    
                return Err(syn::Error::new(item.span(), PARAMETER_TYPE_VALIDATION_MESSAGE));
            };

            let struct_name = &ident;
            let (name, default_value, min_value, max_value, text_to_value, value_to_text) = {
                match new_type {
                    SupportedNewType::Bool => {
                        let BoolParameterAttrs { name, default_value, text_to_value, value_to_text }  = syn::parse(attr)?;
                        (name, default_value, Some(0.0), Some(1.0), text_to_value, value_to_text)
                    },
                    SupportedNewType::F32 => {
                        let ParameterAttrs { name, default_value, min_value, max_value, text_to_value, value_to_text }  = syn::parse::<ParameterAttrs<false, false>>(attr)?;
                        (name, default_value, min_value, max_value, text_to_value, value_to_text)
                    },
                    SupportedNewType::I32 => {
                        let ParameterAttrs { name, default_value, min_value, max_value, text_to_value, value_to_text }  = syn::parse::<ParameterAttrs<true, false>>(attr)?;
                        (name, default_value, min_value, max_value, text_to_value, value_to_text)
                    },
                    SupportedNewType::U32 => {                            
                        let ParameterAttrs { name, default_value, min_value, max_value, text_to_value, value_to_text }  = syn::parse::<ParameterAttrs<true, true>>(attr)?;
                        (name, default_value, min_value, max_value, text_to_value, value_to_text)
                    }
                }
            };

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

            let kind = match new_type {
                SupportedNewType::Bool => quote! { pure_audio::ParameterKind::Bool },
                SupportedNewType::F32 => quote! { pure_audio::ParameterKind::F32 },
                SupportedNewType::I32 => quote! { pure_audio::ParameterKind::I32 },
                SupportedNewType::U32 => quote! { pure_audio::ParameterKind::U32 }
            };

            let from_parameter = match new_type {
                SupportedNewType::Bool => quote! { Self(value == 1) },
                SupportedNewType::F32 => quote! { Self(f32::from_bits(value)) },
                SupportedNewType::I32 => quote! { unsafe { Self(std::mem::transmute(value)) } },
                SupportedNewType::U32 => quote! { Self(value) }
            };

            let f64_to_value = match new_type {
                SupportedNewType::Bool => quote! { d as u32 },
                SupportedNewType::F32 => quote! { (d as f32).to_bits() },
                SupportedNewType::I32 => quote! { unsafe { std::mem::transmute(d as i32) } },
                SupportedNewType::U32 => quote! { d as u32 }
            };

            let value_to_f64 = match new_type {
                SupportedNewType::Bool => quote! { value as f64 },
                SupportedNewType::F32 => quote! { f32::from_bits(value) as f64 },
                SupportedNewType::I32 => quote! { unsafe { std::mem::transmute::<u32, i32>(value) as f64 } },
                SupportedNewType::U32 => quote! { value as f64 },
            };

            let parameter_arithmetic_macro = match new_type {
                SupportedNewType::Bool => quote! { },
                SupportedNewType::F32 | SupportedNewType::I32 | SupportedNewType::U32 => quote! { #[pure_audio::pure_audio_proc_macro::parameter_arithmetic] }
            };
    
            let implementation = quote! {
                #[derive(Copy, Clone)] 
                #parameter_arithmetic_macro
                #s
    
                impl pure_audio::Parameter for #struct_name {
                    const DESCRIPTOR: pure_audio::ParameterDescriptor = pure_audio::ParameterDescriptor {
                        name: #name,
                        default_value: #default_value,
                        min_value: #min_value,
                        max_value: #max_value,
                        kind: #kind
                    };
                    
                    #[inline]
                    fn from_parameter(value: u32) -> Self {
                        #from_parameter
                    }

                    #[inline]
                    fn f64_to_value(d: f64) -> u32 {
                        #f64_to_value
                    }
                    
                    #[inline]
                    fn value_to_f64(value: u32) -> f64 {
                        #value_to_f64
                    }
                    
                    #text_to_value

                    #value_to_text
                }
            };

            Ok(TokenStream::from(implementation))
        },
        Item::Enum(ref e) => {
            let (variant_names, variant_values): (Vec<_>, Vec<_>) = 
                e
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(index, v)| {
                        if v.fields.is_empty() {
                            Ok((v.ident.to_string(), index as f64))
                        } else {
                            Err(syn::Error::new(v.span(), "enum variants with fields are not supported"))
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .unzip();

            if variant_names.is_empty() {
                return Err(syn::Error::new(e.span(), "enum must have at least one variant"))
            }

            let max_value = (variant_names.len() - 1) as f32;
            
            let EnumParameterAttrs { name, default_value, text_to_value, value_to_text } = EnumParameterAttrs::from_input_and_variants(attr, &variant_names)?;

            let enum_name = &e.ident;
            let name = name.unwrap_or(enum_name.to_string());
            let default_value = default_value.unwrap_or(0f32);
            let min_value = 0f32;
            let max_value = max_value;                    

            let text_to_value = if let Some(expr) = text_to_value {
                quote! {
                    #[inline]
                    fn text_to_value(text: &str) -> Option<f64> {
                        #expr(text) 
                    }
                }
            } else {
                let variant_names_lowercase: Vec<_> = 
                    variant_names
                        .iter()
                        .map(|name| name.to_lowercase())
                        .collect();
                quote! {
                    #[inline]
                    fn text_to_value(text: &str) -> Option<f64> {
                        match text.to_lowercase().as_str() {
                            #(#variant_names_lowercase => Some(#variant_values),)*
                            _ => None
                        }
                    }
                }
            };

            let value_to_text = if let Some(expr) = value_to_text {
                quote! { 
                    #[inline]
                    fn value_to_text(value: f64, writer: &mut impl std::fmt::Write) -> bool {
                        #expr(value, writer)
                    } 
                }
            } else {
                quote! { 
                    #[inline]
                    fn value_to_text(value: f64, writer: &mut impl std::fmt::Write) -> bool {
                        let value_str = match value {
                            #(#variant_values => #variant_names,)*
                            _ => return false
                        };
                        write!(writer, "{value_str}").is_ok()
                    }
                }
            };
    
            let implementation = quote! {
                #[derive(Copy, Clone)]
                #[repr(u32)]
                #e
    
                impl pure_audio::Parameter for #enum_name {
                    const DESCRIPTOR: pure_audio::ParameterDescriptor = pure_audio::ParameterDescriptor {
                        name: #name,
                        default_value: #default_value,
                        min_value: #min_value,
                        max_value: #max_value,
                        kind: pure_audio::ParameterKind::Enum(&[
                            #(#variant_names,)*
                        ])
                    };
                    
                    #[inline]
                    fn from_parameter(value: u32) -> Self {
                        unsafe { core::mem::transmute(value) }
                    }

                    #[inline]
                    fn f64_to_value(d: f64) -> u32 {
                        d as u32
                    }
                    
                    #[inline]
                    fn value_to_f64(value: u32) -> f64 {
                        value as f64
                    }
                    
                    #text_to_value

                    #value_to_text
                }
            };

            Ok(TokenStream::from(implementation))
        },
        _ => Err(syn::Error::new(item.span(), "type not supported"))
    }
}

/// Implements arithmetic operators for a parameter.
#[proc_macro_attribute]
pub fn parameter_arithmetic(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = syn::parse::<syn::Item>(input).unwrap();
    if let syn::Item::Struct(ref s) = item {
        let ItemStruct { ident, fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }), .. } = s else {
            return TokenStream::from(syn::Error::new(item.span(), "type must be a tuple struct with a single f32, i32 or u32 field").into_compile_error());
        };

        if unnamed.len() != 1 {
            return TokenStream::from(syn::Error::new(item.span(), "type must be a tuple struct with a single f32, i32 or u32 field").into_compile_error());
        }

        let inner_type = &unnamed[0].ty;

        let name = ident;
        let ops_traits_tokens = [
            ("Add", '+'),
            ("Div", '/'),
            ("Mul", '*'),
            ("Sub", '-')
        ];

        let impls = 
            ops_traits_tokens
                .map(|(op_trait, op_token)| {
                    let op_trait_ident = format_ident!("{op_trait}");
                    let op_fn_ident = format_ident!("{}", op_trait.to_lowercase());
                    let op_token_tree = TokenTree::Punct(Punct::new(op_token, Spacing::Alone));
                    quote! {
                        impl std::ops::#op_trait_ident<#inner_type> for #name {
                            type Output = #inner_type;
                        
                            #[inline]
                            fn #op_fn_ident(self, rhs: #inner_type) -> Self::Output {
                                self.0 #op_token_tree rhs
                            }
                        }
            
                        impl std::ops::#op_trait_ident<&#inner_type> for #name {
                            type Output = #inner_type;
                        
                            #[inline]
                            fn #op_fn_ident(self, rhs: &#inner_type) -> Self::Output {
                                self.0 #op_token_tree rhs
                            }
                        }
            
                        impl std::ops::#op_trait_ident<#name> for #inner_type {
                            type Output = #inner_type;
                        
                            #[inline]
                            fn #op_fn_ident(self, rhs: #name) -> Self::Output {
                                self #op_token_tree rhs.0
                            }
                        }
            
                        impl std::ops::#op_trait_ident<#name> for &#inner_type {
                            type Output = #inner_type;
                        
                            #[inline]
                            fn #op_fn_ident(self, rhs: #name) -> Self::Output {
                                self #op_token_tree rhs.0
                            }
                        }

                        impl std::ops::#op_trait_ident<#inner_type> for &#name {
                            type Output = #inner_type;
                        
                            #[inline]
                            fn #op_fn_ident(self, rhs: #inner_type) -> Self::Output {
                                self.0 #op_token_tree rhs
                            }
                        }
            
                        impl std::ops::#op_trait_ident<&#inner_type> for &#name {
                            type Output = #inner_type;
                        
                            #[inline]
                            fn #op_fn_ident(self, rhs: &#inner_type) -> Self::Output {
                                self.0 #op_token_tree rhs
                            }
                        }
            
                        impl std::ops::#op_trait_ident<&#name> for #inner_type {
                            type Output = #inner_type;
                        
                            #[inline]
                            fn #op_fn_ident(self, rhs: &#name) -> Self::Output {
                                self #op_token_tree rhs.0
                            }
                        }
            
                        impl std::ops::#op_trait_ident<&#name> for &#inner_type {
                            type Output = #inner_type;
                        
                            #[inline]
                            fn #op_fn_ident(self, rhs: &#name) -> Self::Output {
                                self #op_token_tree rhs.0
                            }
                        }
                    }
                });

        let implementation = quote! {
            #s

            #(#impls)*

            impl PartialEq<#inner_type> for #name {                
                #[inline]
                fn eq(&self, other: &#inner_type) -> bool {
                    self.0.eq(other)
                }
            }
            
            impl PartialOrd<#inner_type> for #name {
                #[inline]
                fn partial_cmp(&self, other: &#inner_type) -> Option<std::cmp::Ordering> {
                   self.0.partial_cmp(other)
                }
            }
        };
    
        TokenStream::from(implementation)
    } else {
        TokenStream::from(syn::Error::new(item.span(), "type not supported").into_compile_error())
    }
}