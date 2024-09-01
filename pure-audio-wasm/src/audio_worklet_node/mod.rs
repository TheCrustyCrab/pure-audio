use wasm_bindgen::prelude::*;
use web_sys::{AudioWorkletNode, AudioWorkletNodeOptions, BaseAudioContext, EventTarget};

#[wasm_bindgen(typescript_custom_section)]
const TS_INSTRUMENT_AUDIO_WORKLET_NODE: &'static str = r#"
    export class InstrumentAudioWorkletNode extends AudioWorkletNode {
        noteOn(): void;
        noteOff(): void;
    }
    "#;

#[wasm_bindgen(module = "/src/audio_worklet_node/js/instrumentAudioWorkletNode.js")]
extern "C" {
    #[wasm_bindgen (extends = AudioWorkletNode , extends = EventTarget , extends = :: js_sys :: Object , js_name = InstrumentAudioWorkletNode , typescript_type = "InstrumentAudioWorkletNode")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type InstrumentAudioWorkletNode;
    #[wasm_bindgen(catch, constructor, js_class = "InstrumentAudioWorkletNode")]
    pub fn new_with_options(
        context: &BaseAudioContext,
        name: &str,
        options: &AudioWorkletNodeOptions,
    ) -> Result<InstrumentAudioWorkletNode, JsValue>;
}

pub trait WasmAudioWorkletNode : Sized {
    fn new_with_options(
        context: &BaseAudioContext,
        name: &str,
        options: &AudioWorkletNodeOptions,
    ) -> Result<Self, JsValue>;
}

impl WasmAudioWorkletNode for AudioWorkletNode {
    fn new_with_options(
        context: &BaseAudioContext,
        name: &str,
        options: &AudioWorkletNodeOptions,
    ) -> Result<Self, JsValue> {
        AudioWorkletNode::new_with_options(context, name, options)
    }
}

impl WasmAudioWorkletNode for InstrumentAudioWorkletNode {
    fn new_with_options(
        context: &BaseAudioContext,
        name: &str,
        options: &AudioWorkletNodeOptions,
    ) -> Result<Self, JsValue> {
        InstrumentAudioWorkletNode::new_with_options(context, name, options)
    }
}