pub enum Event {
    NoteOn { key: u8 },
    NoteOff { key: u8 }
}