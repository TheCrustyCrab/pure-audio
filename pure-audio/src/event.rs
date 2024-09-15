pub enum Event {
    NoteOn { key: u8, velocity: u8 },
    NoteOff { key: u8, velocity: u8 }
}