use pure_audio::{OutEvent, OutEventDispatcher};
use wasm_bindgen::JsValue;
use crate::NoteEndEvent;

pub(crate) struct WasmOutEventDispatcher {
    output_event_callback: js_sys::Function
}

impl WasmOutEventDispatcher {
    #[inline]
    pub fn new(output_event_callback: js_sys::Function) -> Self {
        Self {
            output_event_callback
        }
    }
}

impl OutEventDispatcher for WasmOutEventDispatcher {
    fn dispatch(&self, event: OutEvent) {        
        let wasm_out_event = match event {
            OutEvent::NoteEnd { port_index, channel, key, note_id, .. } => {
                NoteEndEvent::new(port_index, channel, key, note_id)
            }
        };
        let _ = self.output_event_callback.call1(&JsValue::NULL, &wasm_out_event);
    }
}