use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, token::Comma, Ident, LitInt};

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
            F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>, #(#generic_idents),*),
            #(
                #generic_idents: 'static + FromParameters,
            )*
            S: 'static + Default,
            {
                #[inline]
                fn process<'a>(
                    &'a mut self,
                    inputs: &'a [[&'a [f32]; NUM_CHANNELS]; NUM_INPUTS],
                    outputs: [[&'a mut [f32]; NUM_CHANNELS]; NUM_OUTPUTS],
                    parameters: &'a [f32; #num_params],
                    events: &'a [Event]
                ) {
                    #(
                        let #generic_idents = #generic_idents::from_parameters(parameters, #indices);
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
            F: 'static + FnMut(AudioData<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, S>, #(#generic_idents,)*),
            #(
                #generic_idents: 'static + FromParameters,
            )*
            S: 'static + Default,
            {
                type Out = ProcessorWrapper<F, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, #num_params, (#(#generic_idents,)*), S>;
                const PARAM_DESCRIPTORS: [ParameterDescriptor; #num_params] = [#(#generic_idents::DESCRIPTOR),*];

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