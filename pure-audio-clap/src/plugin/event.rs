use clap_sys::events::{
    clap_event_header, clap_event_note, clap_output_events, CLAP_CORE_EVENT_SPACE_ID,
    CLAP_EVENT_NOTE_END,
};
use pure_audio::{OutEvent, OutEventDispatcher};

pub(crate) struct ClapOutEventDispatcher {
    clap_out_events: *const clap_output_events,
    frames_count: u32,
}

impl ClapOutEventDispatcher {
    #[inline]
    pub(crate) fn new(clap_out_events: *const clap_output_events, frames_count: u32) -> Self {
        Self {
            clap_out_events,
            frames_count,
        }
    }
}

impl OutEventDispatcher for ClapOutEventDispatcher {
    fn dispatch(&self, event: OutEvent) {
        unsafe {
            let out_events = &*self.clap_out_events;
            let clap_out_event = match event {
                OutEvent::NoteEnd {
                    port_index,
                    channel,
                    key,
                    note_id,
                    velocity,
                } => clap_event_note {
                    header: clap_event_header {
                        flags: 0,
                        size: std::mem::size_of::<clap_event_note>() as u32,
                        space_id: CLAP_CORE_EVENT_SPACE_ID,
                        time: self.frames_count - 1,
                        type_: CLAP_EVENT_NOTE_END,
                    },
                    channel: channel as i16,
                    key: key as i16,
                    note_id,
                    port_index: port_index as i16,
                    velocity: velocity as f64,
                },
            };
            out_events.try_push.unwrap()(self.clap_out_events, &clap_out_event.header);
        }
    }
}
