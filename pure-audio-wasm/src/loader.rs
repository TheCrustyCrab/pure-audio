use crate::{es_module::{ImportMeta, IMPORT_META}, PureAudioWorkletNode, parameter::WasmParameterConverter, PROCESSOR_BLOCK_LENGTH};
use js_sys::{Array, Map, Object, Reflect};
use pure_audio::{AutomationRate, IntoProcessor, ParameterDescriptor, ParameterKind};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;
use web_sys::{console::log_1, window, AudioContext, AudioParam, AudioWorkletNodeOptions, Blob, BlobPropertyBag, ChannelCountMode, HtmlInputElement, HtmlLabelElement, MessagePort, Url};

const AUDIO_CONTEXT_REGISTERED_MODULES_FIELD_NAME: &'static str = "registeredModules";

pub async fn register_and_create_node<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, A, Params, S, P>(name: &str, 
    process: P, ctx: &AudioContext, generate_parameter_ui_div_id: Option<&str>)
-> Result<PureAudioWorkletNode, JsValue>
where
    P: IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    log_1(&"Checking registered modules".into());
    let registered_modules = {
        if let Ok(registered_modules) = Reflect::get(ctx, &AUDIO_CONTEXT_REGISTERED_MODULES_FIELD_NAME.into()).and_then(JsCast::dyn_into::<Array>) {
            registered_modules
        } else {
            let registered_modules = Array::new();
            Reflect::set(ctx, &AUDIO_CONTEXT_REGISTERED_MODULES_FIELD_NAME.into(), &registered_modules).unwrap_throw();
            registered_modules
        }
    };

    if registered_modules.find(&mut |element, _, _| element == JsValue::from(name)).is_undefined() {
        register_node(name, &process, ctx).await?;
        registered_modules.push(&name.into());
    }

    let audio_worklet_node = create_node(name, NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, ctx, process)?;

    if let Some(div_id) = generate_parameter_ui_div_id {
        let window = window().unwrap();
        let document = window.document().unwrap();
        let control = document.create_element("div")?;
        let param_map = audio_worklet_node.parameters().unwrap();
        let port = audio_worklet_node.port().unwrap();

        fn add_parameter_input_change_event_handler(input_element: &HtmlInputElement, parameter: AudioParam, port: &MessagePort, map_f32: impl Fn(&HtmlInputElement) -> f32 + 'static) 
        -> Result<(), JsValue> {
            let port = port.clone();
            let closure = Closure::<dyn Fn(_)>::new(move |event: web_sys::Event| {
                let input_element = event.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
                let value = map_f32(&input_element);
                parameter.set_value(value);
                let msg = Object::new();
                let _ = Reflect::set(&msg, &"type".into(), &"indicateParamsChanged".into());
                let _ = port.post_message(&msg);
            });

            input_element.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;

            // rely on weak references and the JS GC to drop the closure
           closure.forget();

           Ok(())
        }

        fn add_float_parameter_input_change_event_handler(index: usize, input_element: & HtmlInputElement,
            parameter: AudioParam, port: &MessagePort,
            map_f32: impl Fn(&HtmlInputElement) -> f32 + 'static, parameter_converter_ptr: usize)
        -> Result<(), JsValue> {
            let port = port.clone();
            let closure = Closure::<dyn Fn(_)>::new(move |event: web_sys::Event| {
                let input_element = event.target().unwrap().dyn_into::<HtmlInputElement>().unwrap();
                let value = map_f32(&input_element);
                parameter.set_value(value);
                let parameter_converter = WasmParameterConverter::from_raw_ptr(parameter_converter_ptr);
                let text = parameter_converter.value_to_text(index, value as f64);
                let label_element = input_element.next_element_sibling().unwrap();
                label_element.set_text_content(Some(&text));
                let msg = Object::new();
                let _ = Reflect::set(&msg, &"type".into(), &"indicateParamsChanged".into());
                let _ = port.post_message(&msg);
            });
            input_element.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;

            // rely on weak references and the JS GC to drop the closure
           closure.forget();

           Ok(())
        }
        
        for (index, &(ParameterDescriptor { name, default_value, min_value, max_value, kind }, ..)) in P::PARAM_DESCRIPTORS.iter().enumerate() {
            let paragraph = document.create_element("p")?;
            paragraph.set_text_content(Some(&format!("{name}:")));
            match kind {
                ParameterKind::Bool => {
                    let checkbox = document.create_element("input")?.dyn_into::<HtmlInputElement>()?;
                    checkbox.set_type("checkbox");
                    checkbox.set_checked(default_value == 1f32);
                    let parameter = param_map.get(name).unwrap();
                    add_parameter_input_change_event_handler(&checkbox, parameter, &port, |input| if input.checked() { 1f32 } else { 0f32 })?;
                    paragraph.append_child(&checkbox)?;
                },
                ParameterKind::Enum(variants) => {
                    for (value, option) in variants.iter().enumerate() {
                        let radio = document.create_element("input")?.dyn_into::<HtmlInputElement>()?;
                        radio.set_type("radio");
                        radio.set_name(name);
                        radio.set_id(option);
                        radio.set_value(&format!("{value}"));
                        radio.set_checked(value as f32 == default_value);
                        let label = document.create_element("label")?.dyn_into::<HtmlLabelElement>()?;
                        label.set_html_for(option);
                        label.set_inner_html(option);
                        let parameter = param_map.get(name).unwrap();
                        add_parameter_input_change_event_handler(&radio, parameter, &port, |input| input.value().parse().unwrap())?;
                        paragraph.append_child(&radio)?;
                        paragraph.append_child(&label)?;
                    }
                },
                ParameterKind::F32 | ParameterKind::I32 | ParameterKind::U32 => {
                    let slider = document.create_element("input")?.dyn_into::<HtmlInputElement>()?;
                    slider.set_type("range");
                    slider.set_min(&min_value.to_string());
                    slider.set_max(&max_value.to_string());
                    let step = match kind {
                        ParameterKind::Bool | ParameterKind::Enum(_) | ParameterKind::I32 | ParameterKind::U32 => "1",
                        ParameterKind::F32 => "0.01"
                    };
                    slider.set_step(step);
                    slider.set_value(&default_value.to_string());
                    paragraph.append_child(&slider)?;
                    let parameter_converter_ptr = audio_worklet_node.get_raw_parameter_converter_ptr();
                    let parameter_converter = WasmParameterConverter::from_raw_ptr(parameter_converter_ptr);
                    let text = parameter_converter.value_to_text(index, default_value as f64);
                    let label_element = document.create_element("div")?;
                    label_element.set_text_content(Some(&text));
                    paragraph.append_child(&label_element)?;
                    let parameter = param_map.get(name).unwrap();
                    let _ = add_float_parameter_input_change_event_handler(index, &slider, parameter, &port,
                        |input| input.value_as_number() as f32, parameter_converter_ptr);
                }
            }
            control.append_child(&paragraph)?;
        }
        let div = document.get_element_by_id(div_id).ok_or(format!("div #{div_id} not found"))?;
        div.set_inner_html("");
        div.append_child(&control)?;
    }

    Ok(audio_worklet_node)
}

async fn register_node<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, P, A, Params, S>(
    name: &str, _process: &P, ctx: &AudioContext) -> Result<(), JsValue>
where
    P: IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S>
{
    log_1(&"Registering node".into());
    let meta_url: String = IMPORT_META.with(ImportMeta::url).into();
    log_1(&format!("Meta url: {meta_url}").into());

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
                .flat_map(|input_index| {
                    (0..NUM_CHANNELS)
                        .map(move |channel_index| {
                            let offset = input_index * NUM_CHANNELS * PROCESSOR_BLOCK_LENGTH + channel_index * PROCESSOR_BLOCK_LENGTH;
                            format!("this.float32Memory.set(inputs[{input_index}][{channel_index}] || new Float32Array({PROCESSOR_BLOCK_LENGTH}), this.inputsPtr + {offset});")
                        })
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    let process_copy_output = 
        (0..NUM_OUTPUTS)
            .flat_map(|output_index| {
                (0..NUM_CHANNELS)
                    .map(move |channel_index| {
                        let offset = output_index * NUM_CHANNELS * PROCESSOR_BLOCK_LENGTH + channel_index * PROCESSOR_BLOCK_LENGTH;
                        format!("outputs[{output_index}][{channel_index}].set(this.float32Memory.subarray(this.outputsPtr + {offset}, this.outputsPtr + {offset} + {PROCESSOR_BLOCK_LENGTH}));")
                    })
            })
            .collect::<Vec<_>>()
            .join("\n");

    let process_copy_parameters_per_sample = 
        P::PARAM_DESCRIPTORS
            .iter()
            .enumerate()
            .filter(|(.., (.., automation_rate))| if let AutomationRate::A = automation_rate { true } else { false })
            .map(|(i, (desc, ..))| {
                let name = desc.name;
                let data_offset = std::mem::align_of::<Option<[f32; PROCESSOR_BLOCK_LENGTH]>>() / 4;
                let offset = data_offset + i * (PROCESSOR_BLOCK_LENGTH + data_offset);
                // for a-rate parameters, the array will only contain multiple (128) values when necessary (e.g. during a linear ramp)
                let memory = match desc.kind {
                    ParameterKind::Bool | ParameterKind::Enum(_) | ParameterKind::U32 => "uint32Memory",
                    ParameterKind::F32 => "float32Memory",
                    ParameterKind::I32 => "int32Memory"                     
                };
                format!(
                    r#"
                        if (parameters['{name}'].length > 1) {{ 
                            this.{memory}.set(parameters['{name}'], this.parametersPerSamplePtr + {offset});
                        }} else {{
                            this.{memory}.fill(parameters['{name}'][0], this.parametersPerSamplePtr + {offset}, this.parametersPerSamplePtr + {offset} + {PROCESSOR_BLOCK_LENGTH});
                        }}
                    "#
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

    let (parameter_descriptors, parameter_copies): (Vec<_>, Vec<_>) = 
        P::PARAM_DESCRIPTORS
            .iter()
            .enumerate()
            .map(|(index, &(ParameterDescriptor { name, default_value, min_value, max_value, kind }, automation_rate))| {
                (format!(
                    r#"{{
                        name: '{name}',
                        defaultValue: {default_value},
                        minValue: {min_value},
                        maxValue: {max_value},
                        automationRate: '{automation_rate}'
                    }}
                    "#
                ), {
                        let memory = match kind {
                            ParameterKind::Bool | ParameterKind::Enum(_) | ParameterKind::U32 => "uint32Memory",
                            ParameterKind::F32 => "float32Memory",
                            ParameterKind::I32 => "int32Memory"                     
                        };
                        format!("this.{memory}[this.parametersPtr + {index}] = parameters['{name}'][0];")
                })
            })
            .unzip();

    let (parameter_descriptors, parameter_copies) = 
        (parameter_descriptors.join(", "), parameter_copies.join("\n"));
    
    // available global variables: sampleRate, currentTime, currentFrame
    // see https://developer.mozilla.org/en-US/docs/Web/API/AudioWorkletGlobalScope
    let code = format!(
        r#"
        import {{ initSync, createWasmProcessor }} from '{meta_url}';

        registerProcessor("{name}", class {name} extends AudioWorkletProcessor {{
            constructor(options) {{
                // debugger;
                super();
                this.port.onmessage = msg => {{
                    if (msg.data.type === "noteOn") {{
                        this.processor.note_on(msg.data.data.key, msg.data.data.velocity);
                    }} else if (msg.data.type === "noteOff") {{
                        this.processor.note_off(msg.data.data.key, msg.data.data.velocity);
                    }} else if (msg.data.type === "scheduleNoteOn") {{
                        this.processor.schedule_note_on(msg.data.data.time, msg.data.data.key, msg.data.data.velocity);
                    }} else if (msg.data.type === "scheduleNoteOff") {{
                        this.processor.schedule_note_off(msg.data.data.time, msg.data.data.key, msg.data.data.velocity);
                    }} else if (msg.data.type === "indicateParamsChanged") {{
                        this.processor.indicate_params_changed();
                    }} else if (msg.data.type === "requestStop") {{
                        this.stopRequested = true;
                    }}
                }};
                const [module] = options.processorOptions;
                const {{ memory }} = initSync({{ module }});
                const onOutputEvent = event => this.port.postMessage({{
                    type: "outputEvent",
                    data: event
                }});
                this.processor = createWasmProcessor(sampleRate, onOutputEvent);

                this.inputsPtr = this.processor.get_inputs_ptr() / 4; // NUM_INPUTS * NUM_CHANNELS * [f32; 128]
                this.outputsPtr = this.processor.get_outputs_ptr() / 4; // NUM_OUTPUTS * NUM_CHANNELS * [f32; 128]
                this.parametersPtr = this.processor.get_parameters_ptr() / 4;
                this.parametersPerSamplePtr = this.processor.get_parameters_per_sample_ptr() / 4; // NUM_PARAMS * Option<[f32; 128]>
                this.float32Memory = new Float32Array(memory.buffer);
                this.uint32Memory = new Uint32Array(memory.buffer);
                this.int32Memory = new Int32Array(memory.buffer);
                this.stopRequested = false;
                this.lastTime = 0;
                this.allowedProcessingTime = Math.round(1/375 * 100000000) / 100000000; // 375 blocks of 128 samples at 48000 kHz
            }}

            process(inputs, outputs, parameters) {{
                {process_condition}
                {process_copy_input}
                {parameter_copies}
                {process_copy_parameters_per_sample}
                this.processor.process();
                {process_copy_output}
                const processingTime = currentTime - this.lastTime;
                if (processingTime > this.allowedProcessingTime)
                    console.log("processing took too long: " + processingTime);
                this.lastTime = currentTime;
                return !this.stopRequested; // todo: take tail time of synths into account
            }}

            static get parameterDescriptors() {{
                return [
                    {parameter_descriptors}
                ];
            }}     
        }});
    "#
    );

    let options = BlobPropertyBag::new();
    options.set_type("text/javascript");
    let blob =
        Blob::new_with_str_sequence_and_options(&Array::of1(&JsValue::from_str(&code)), &options)?;
    let url = Url::create_object_url_with_blob(&blob)?;

    log_1(&format!("Blob url: {url}").into());

    JsFuture::from(ctx.audio_worklet()?.add_module(&url)?).await?;
    log_1(&"Added module".into());
    Ok(())
}

fn create_node<const NUM_INPUTS: usize, const NUM_OUTPUTS: usize, const NUM_CHANNELS: usize, const NUM_PARAMS: usize, P, A, Params, S>(name: &str, num_inputs: usize, num_outputs: usize, num_channels: usize, ctx: &AudioContext, _process: P) -> Result<PureAudioWorkletNode, JsValue>
where
    P: IntoProcessor<NUM_INPUTS, NUM_OUTPUTS, NUM_CHANNELS, NUM_PARAMS, A, Params, S> {
    log_1(&"Creating node".into());
    let options = AudioWorkletNodeOptions::new();
    options.set_number_of_inputs(num_inputs as u32);
    options.set_number_of_outputs(num_outputs as u32);
    options.set_channel_count(num_channels as u32);
    options.set_channel_count_mode(ChannelCountMode::Explicit);
    let output_channel_counts = Array::new_with_length(num_outputs as u32);
    for i in 0..num_outputs as u32 {
        output_channel_counts.set(i, num_channels.into());
    }
    options.set_output_channel_count(&output_channel_counts);
    options.set_processor_options(Some(
        &Array::of1(&wasm_bindgen::module())
    ));

    let parameter_name_index_map = Map::new();
    for (index, &(ParameterDescriptor { name, .. }, ..)) in P::PARAM_DESCRIPTORS.iter().enumerate() {
        parameter_name_index_map.set(&name.into(), &index.into());
    }

    PureAudioWorkletNode::new_with_options(&ctx, name, &options, wasm_bindgen::exports(), parameter_name_index_map)
}