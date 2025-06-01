use js_sys::JsString;
use pure_audio::{OutEvent, OutEventDispatcher};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use crate::NoteEvent;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(thread_local_v2, static_string)]
    static NOTE_EVENT_END: JsString = "end";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static NOTE_EVENT_SCHEDULE_OFF: JsString = "scheduleOff";
    #[wasm_bindgen(thread_local_v2, static_string)]
    static NOTE_EVENT_SCHEDULE_ON: JsString = "scheduleOn";
}

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

    #[inline]
    pub fn dispatch_note_event(&self, event: NoteEvent) {
        let _ = self.output_event_callback.call1(&JsValue::NULL, &event);
    }

    #[inline]
    pub fn dispatch_note_schedule_off_event(&self, key: u8) {
        let event = NoteEvent::new(NOTE_EVENT_SCHEDULE_OFF.with(JsString::clone), 0, 0, key, 0);
        let _ = self.output_event_callback.call1(&JsValue::NULL, &event);
    }

    #[inline]
    pub fn dispatch_note_schedule_on_event(&self, key: u8) {
        let event = NoteEvent::new(NOTE_EVENT_SCHEDULE_ON.with(JsString::clone), 0, 0, key, 0);
        let _ = self.output_event_callback.call1(&JsValue::NULL, &event);
    }
}

impl OutEventDispatcher for WasmOutEventDispatcher {
    fn dispatch(&self, event: OutEvent) {        
        let wasm_out_event = match event {
            OutEvent::NoteEnd { port_index, channel, key, note_id, .. } => {
                NoteEvent::new(NOTE_EVENT_END.with(JsString::clone), port_index, channel, key, note_id)
            }
        };
        self.dispatch_note_event(wasm_out_event);
    }
}