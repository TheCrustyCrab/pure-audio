pub enum Event {
    NoteOn { port_index: i32, channel: i32, key: u8, note_id: i32, velocity: u8 },
    NoteOff { port_index: i32, channel: i32, key: u8, note_id: i32, velocity: u8 },
    ParamsChanged
}

pub enum OutEvent {
    NoteEnd { port_index: i32, channel: i32, key: u8, note_id: i32, velocity: u8 }
}

pub trait OutEventDispatcher {
    fn dispatch(&self, event: OutEvent);
}

pub struct OutEvents<'a> {
    dispatcher: &'a dyn OutEventDispatcher
}

impl<'a> OutEvents<'a> {
    #[inline]
    pub fn new(dispatcher: &'a dyn OutEventDispatcher) -> Self {
        Self {
            dispatcher
        }
    }

    #[inline]
    pub fn push(&self, event: OutEvent) {
        self.dispatcher.dispatch(event);
    }
}