use crate::{audio_worklet_node::WasmAudioWorkletNode, es_module::{ImportMeta, IMPORT_META}, IntoWasmProcessor, PROCESSOR_BLOCK_LENGTH};
use js_sys::{Array, Promise, Reflect, WebAssembly};
use pure_audio::ParameterDescriptor;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioContext, AudioWorkletNodeOptions, Blob, BlobPropertyBag, Request, RequestInit, Url};

const AUDIO_CONTEXT_REGISTERED_MODULES_FIELD_NAME: &'static str = "registeredModules";

pub async fn register_and_create_node<const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, Params, S, F>(name: &str, 
    process: F, ctx: &AudioContext)
-> Result<F::AudioWorkletNodeType, JsValue>
where
    F: IntoWasmProcessor<IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    let registered_modules = {
        if let Ok(registered_modules) = Reflect::get(ctx, &AUDIO_CONTEXT_REGISTERED_MODULES_FIELD_NAME.into()).and_then(JsCast::dyn_into::<Array>) {
            registered_modules
        } else {
            let registered_modules = Array::new();
            Reflect::set(ctx, &AUDIO_CONTEXT_REGISTERED_MODULES_FIELD_NAME.into(), &registered_modules).unwrap();
            registered_modules
        }
    };

    if registered_modules.find(&mut |element, _, _| element == JsValue::from(name)).is_undefined() {
        register_node(name, &process, ctx).await?;
        registered_modules.push(&name.into());
    }

    create_node(name, &process, ctx).await
}

async fn register_node<const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, F, Params, S>(
    name: &str, _process: &F, ctx: &AudioContext) -> Result<(), JsValue>
where
    F: IntoWasmProcessor<IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    let meta_url: String = IMPORT_META.with(ImportMeta::url).into();
    let mut parts = meta_url.split("/").collect::<Vec<_>>();
    let bindgen_file = format!("{name}.js");
    *parts.iter_mut().last().unwrap() = &bindgen_file;
    let meta_url = parts.join("/");

    let (process_condition, process_copy_input) = 
    if NUM_INPUTS == 0 {
        (
            "if (outputs[0].length < 1) return true;", // not sure if needed?
            String::new() // no input
        )
    } 
    else {    
        (
            "if (inputs.every(i => i.length === 0) || outputs[0].length < 1) return true;",
            (0..NUM_INPUTS)
                .map(|i| {
                    let offset = i * NUM_CHANNELS * PROCESSOR_BLOCK_LENGTH;
                    format!("this.float32Memory.set(inputs[{i}][0] || new Float32Array({PROCESSOR_BLOCK_LENGTH}), this.inputsPtr + {offset});")
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    let process_copy_output = 
        (0..NUM_OUTPUTS)
            .map(|i| {
                let offset = i * NUM_CHANNELS * PROCESSOR_BLOCK_LENGTH;
                format!("outputs[{i}][0].set(this.float32Memory.subarray(this.outputsPtr + {offset}, this.outputsPtr + {offset} + {PROCESSOR_BLOCK_LENGTH}));")
            })
            .collect::<Vec<_>>()
            .join("\n");

    let (parameter_descriptors, parameter_values): (Vec<_>, Vec<_>) = 
        F::get_parameter_descriptors()
            .iter()
            .map(|&ParameterDescriptor { name, default_value, min_value, max_value, automation_rate }| {
                (format!(
                    r#"{{
                        name: '{name}',
                        defaultValue: {default_value},
                        minValue: {min_value},
                        maxValue: {max_value},
                        automationRate: '{automation_rate}'
                    }}
                    "#
                ), format!("parameters['{name}'][0]")) // assume k-rate parameters for now
            })
            .unzip();

    let (parameter_descriptors, parameter_values) = 
        (parameter_descriptors.join(", "), parameter_values.join(", "));
    
    // available global variables: sampleRate, currentTime, currentFrame
    // see https://developer.mozilla.org/en-US/docs/Web/API/AudioWorkletGlobalScope
    let code = format!(
        r#"
        import {{ initSync, create_wasm_processor }} from '{meta_url}';

        registerProcessor("{name}", class {name} extends AudioWorkletProcessor {{
            constructor(options) {{
                // debugger;
                super();
                this.port.onmessage = msg => {{
                    console.log("Audio thread received message: ");
                    console.log(msg);
                    this.port.postMessage("hello from audio thread!");
                    if (msg.data.type === "noteOn") {{
                        this.processor.note_on(msg.data.data.key, msg.data.data.velocity);
                    }} else if (msg.data.type === "noteOff") {{
                        this.processor.note_off(msg.data.data.key, msg.data.data.velocity);
                    }}
                }};
                const [module, sampleRate] = options.processorOptions;
                const {{ memory }} = initSync({{ module }});
                this.processor = create_wasm_processor(sampleRate);

                this.inputsPtr = this.processor.get_inputs_ptr() / 4; // NUM_INPUTS * NUM_CHANNELS * [f32; 128]
                this.outputsPtr = this.processor.get_outputs_ptr() / 4; // NUM_OUTPUTS * NUM_CHANNELS * [f32; 128]
                this.parametersPtr = this.processor.get_parameters_ptr() / 4;
                this.float32Memory = new Float32Array(memory.buffer);
            }}

            process(inputs, outputs, parameters) {{
                {process_condition}
                {process_copy_input}
                const flatParameters = [{parameter_values}];
                this.float32Memory.set(new Float32Array(flatParameters), this.parametersPtr);
                this.processor.process();
                {process_copy_output}
                return true;
            }}

            static get parameterDescriptors() {{
                return [
                    {parameter_descriptors}
                ];
            }}     
        }});
    "#
    );

    let mut options = BlobPropertyBag::new();
    options.type_("text/javascript");
    let blob =
        Blob::new_with_str_sequence_and_options(&Array::of1(&JsValue::from_str(&code)), &options)?;
    let url = Url::create_object_url_with_blob(&blob)?;

    JsFuture::from(ctx.audio_worklet()?.add_module(&url)?).await?;
    Ok(())
}

async fn create_node<const IS_INSTRUMENT: bool, const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, F, Params, S>(
    name: &str,
    _process: &F,
    ctx: &AudioContext)
-> Result<F::AudioWorkletNodeType, JsValue>
where
    F: IntoWasmProcessor<IS_INSTRUMENT, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, Params, S>
{
    let mut options = AudioWorkletNodeOptions::new();
    let response = fetch(&format!("{name}_bg.wasm")).await.unwrap();
    let module = JsFuture::from(WebAssembly::compile_streaming(&response)).await.unwrap();

    options.number_of_inputs(NUM_INPUTS as u32);
    options.number_of_outputs(NUM_OUTPUTS as u32);
    options.processor_options(Some(
        &Array::of2(&module, &ctx.sample_rate().into())
    ));
    F::AudioWorkletNodeType::new_with_options(&ctx, name, &options)
}

async fn fetch(url: &str) -> Result<Promise, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");

    let request = Request::new_with_str_and_init(url, &opts)?;

    let window = web_sys::window().unwrap();
    Ok(window.fetch_with_request(&request))
}