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
    ) -> Result<PureAudioWorkletNode, JsValue>;
}