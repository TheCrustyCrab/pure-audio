use wasm_bindgen::prelude::*;
use web_sys::{AudioWorkletNode, AudioWorkletNodeOptions, BaseAudioContext, EventTarget};

#[wasm_bindgen(typescript_custom_section)]
const TS_PURE_AUDIO_WORKLET_NODE: &'static str = include_str!("js/pureAudioWorkletNode.d.ts");

#[wasm_bindgen(module = "/src/audio_worklet_node/js/pureAudioWorkletNode.js")]
extern "C" {
    #[wasm_bindgen (extends = AudioWorkletNode , extends = EventTarget , extends = :: js_sys :: Object , js_name = PureAudioWorkletNode , typescript_type = "PureAudioWorkletNode")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type PureAudioWorkletNode;
    #[wasm_bindgen(catch, constructor, js_class = "PureAudioWorkletNode")]
    pub fn new_with_options(
        context: &BaseAudioContext,
        name: &str,
        options: &AudioWorkletNodeOptions,
        parameter_value_to_text: JsValue
    ) -> Result<PureAudioWorkletNode, JsValue>;
}

// implementing note event types in javascript due to the limited available API in the AudioWorkletGlobalScope (no TextDecoder so no serialization)
#[wasm_bindgen(typescript_custom_section)]
const TS_NOTE_END_EVENT: &'static str = include_str!("js/noteEndEvent.d.ts");

#[wasm_bindgen(module = "/src/audio_worklet_node/js/noteEndEvent.js")]
extern "C" {
    #[wasm_bindgen (extends = EventTarget , extends = :: js_sys :: Object , js_name = NoteEndEvent , typescript_type = "NoteEndEvent")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type NoteEndEvent;
    #[wasm_bindgen(constructor, js_class = "NoteEndEvent")]
    pub fn new(
        port_index: i32,
        channel: i32,
        key: u8,
        note_id: i32,
    ) -> NoteEndEvent;
}