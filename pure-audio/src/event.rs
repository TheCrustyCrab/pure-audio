pub enum Event {
    NoteOn { port_index: i32, channel: i32, key: u8, note_id: i32, velocity: u8 },
    NoteOff { port_index: i32, channel: i32, key: u8, note_id: i32, velocity: u8 },
    ParamsChanged
}

pub enum OutEvent {
    NoteEnd { port_index: i32, channel: i32, key: u8, note_id: i32, velocity: u8 }
}