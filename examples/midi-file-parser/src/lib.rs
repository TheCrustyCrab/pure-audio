use std::io::Read;
use js_sys::JsString;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_MIDI_EVENT: &'static str = include_str!("js/simpleMidi.d.ts");

#[wasm_bindgen(module = "/src/js/simpleMidi.js")]
extern "C" {
    #[wasm_bindgen (extends = :: js_sys :: Object , js_name = SimpleMidiEvent , typescript_type = "SimpleMidiEvent")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SimpleMidiEvent;
    #[wasm_bindgen(constructor, js_class = "SimpleMidiEvent")]
    pub fn new(
        time: f32,
        event_type: JsString,
        key: u8,
        velocity: u8,
    ) -> SimpleMidiEvent;
    
    #[wasm_bindgen (extends = :: js_sys :: Object , js_name = SimpleMidiTimeSignature , typescript_type = "SimpleMidiTimeSignature")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SimpleMidiTimeSignature;
    #[wasm_bindgen(constructor, js_class = "SimpleMidiTimeSignature")]
    pub fn new(
        numerator: u8,
        denominator: u8
    ) -> SimpleMidiTimeSignature;
    
    #[wasm_bindgen (extends = :: js_sys :: Object , js_name = SimpleMidiTrack , typescript_type = "SimpleMidiTrack")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type SimpleMidiTrack;
    #[wasm_bindgen(constructor, js_class = "SimpleMidiTrack")]
    pub fn new(
        tempo: Option<f32>,
        beats: u32,
        time_signature: Option<SimpleMidiTimeSignature>,
        events: Vec<SimpleMidiEvent>
    ) -> SimpleMidiTrack;
}

/// Currently a far from complete midi file parsing implementation.
/// It allows the web demo to load a simple midi file and interpret its note on/off events.
#[wasm_bindgen(js_name = midiToSimpleTracks)]
pub fn midi_to_simple_tracks(mut bytes: &[u8]) -> Result<Vec<SimpleMidiTrack>, JsValue> {
    let midi = Midi::from_reader(&mut bytes)?;
    Ok(midi.to_simple_midi_tracks())
}

const MIDI_HEADER_PREFIX: [u8; 4] = [0x4D, 0x54, 0x68, 0x64]; // MThd
const MIDI_TRACK_PREFIX: [u8; 4] = [0x4D, 0x54, 0x72, 0x6B]; // MTrk

#[derive(Debug)]
enum Format {
    SingleTrack,
    MultiTrack,
    MultiSong
}

impl TryFrom<u16> for Format {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::SingleTrack),
            1 => Ok(Self::MultiTrack),
            2 => Ok(Self::MultiSong),
            _ => Err("Invalid format")
        }
    }
}

#[derive(Debug)]
struct Midi {
    header: Header,
    tracks: Tracks
}

#[derive(Debug)]
struct Header {
    format: Format,
    num_tracks: u16,
    division: u16
}

impl Header {
    fn from_reader(reader: &mut impl Read) -> Result<Self, &'static str> {      
        let mut buf_2_bytes = [0u8; 2];
        let mut buf_4_bytes = [0u8; 4];
        reader.read_exact(&mut buf_4_bytes).or(Err("Incorrect header"))?;
        if buf_4_bytes != MIDI_HEADER_PREFIX {
            return Err("Incorrect header");
        }

        reader.read_exact(&mut buf_4_bytes).or(Err("Incorrect header"))?;
        let header_length = u32::from_be_bytes(buf_4_bytes);
        if header_length != 6 {
            return Err("Incorrect header length, expected 6");
        }

        reader.read_exact(&mut buf_2_bytes).or(Err("Incorrect header"))?;
        let format = Format::try_from(u16::from_be_bytes(buf_2_bytes))?;
        
        reader.read_exact(&mut buf_2_bytes).or(Err("Incorrect header"))?;
        let number_of_tracks = u16::from_be_bytes(buf_2_bytes);
        
        reader.read_exact(&mut buf_2_bytes).or(Err("Incorrect header"))?;
        let division = u16::from_be_bytes(buf_2_bytes); // voorlopig altijd als positief beschouwen: ticks per beat

        Ok(Header { division, format, num_tracks: number_of_tracks })
    }
}

#[derive(Debug)]
struct Tracks(Vec<Track>);

impl Tracks {
    fn from_reader(reader: &mut impl Read, number_of_tracks: usize) -> Result<Self, &'static str> {
        let tracks = 
            (0..number_of_tracks)
                .map(|_| Track::from_reader(reader))
                .collect::<Result<Vec<_>, &'static str>>()?;
        Ok(Self(tracks))
    }
}

#[derive(Debug)]
struct Track {
    track_events: Vec<TrackEvent>
}

impl Track {
    fn from_reader(reader: &mut impl Read) -> Result<Self, &'static str> {
        let mut buf_4_bytes = [0u8; 4];
        reader.read_exact(&mut buf_4_bytes).or(Err("Incorrect track"))?;
        if buf_4_bytes != MIDI_TRACK_PREFIX {
            return Err("Incorrect track");
        }
        
        reader.read_exact(&mut buf_4_bytes).or(Err("Incorrect track"))?;
        let track_data_length = u32::from_be_bytes(buf_4_bytes);

        let mut track_events = vec![];
        let mut track_bytes_read = 0;
        while track_bytes_read < track_data_length {
            let (event_bytes_read, event) = TrackEvent::from_reader(reader)?;
            track_bytes_read += event_bytes_read;
            track_events.push(event);
        }

        Ok(Self { track_events })
    }
}

#[derive(Debug)]
struct TrackEvent {
    v_time: u32,
    event: Event
}

impl TrackEvent {
    fn from_reader(reader: &mut impl Read) -> Result<(u32, Self), &'static str> {
        let mut bytes_read = 0;
        let (value_bytes_read, v_time) = read_variable_length_value(reader).or(Err("Incorrect track"))?;
        bytes_read += value_bytes_read;

        let (event_bytes_read, event) = Event::from_reader(reader)?;
        bytes_read += event_bytes_read;

        Ok((bytes_read, TrackEvent { v_time, event }))
    }
}

#[derive(Debug)]
enum Event {
    Meta(MetaEvent),
    Midi(MidiEvent),
    Sysex
}

#[derive(Debug)]
enum MetaEvent {
    SequenceNumber,
    TextEvent,
    CopyrightNotice,
    SequenceOrTrackName,
    InstrumentName,
    LyricText,
    MarkerText,
    CuePoint,
    MidiChannelPrefixAssignment,
    EndOfTrack,
    TempoSetting(f32),
    SmpteOffset,
    TimeSignature((u8, u8)),
    KeySignature,
    SequencerSpecificEvent
}

impl MetaEvent {
    fn from_reader(reader: &mut impl Read) -> Result<(u32, Self), &'static str> {
        let mut buf_1_byte = [0u8; 1];
        let mut buf_4_bytes = [0u8; 4];
        let mut bytes_read = 0;

        reader.read_exact(&mut buf_1_byte).or(Err("Incorrect meta event"))?;
        bytes_read += 1;
        let (value_bytes_read, event_length) = read_variable_length_value(reader).or(Err("Invalid meta event"))?;
        bytes_read += value_bytes_read;
        let [meta_type] = buf_1_byte;
        match meta_type {
            0x00 => {
                // sequence number
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::SequenceNumber))
            },
            0x01 => {
                // text event
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::TextEvent))
            },
            0x02 => {
                // copyright notice
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::CopyrightNotice))
            },
            0x03 => {
                // sequence or track name
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::SequenceOrTrackName))
            },
            0x04 => {
                // instrument name
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::InstrumentName))
            },
            0x05 => {
                // lyric text
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::LyricText))
            },
            0x06 => {
                // marker text
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::MarkerText))
            },
            0x07 => {
                // cue point
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::CuePoint))
            },
            0x20 => {
                // MIDI channel prefix assignment
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::MidiChannelPrefixAssignment))
            },
            0x2F => {
                // end of track
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::EndOfTrack))
            },
            0x51 => {
                // tempo setting
                if event_length != 3 {
                    return Err("Incorrect tempo setting");
                }
                
                buf_4_bytes[0] = 0;
                for i in 1..=event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Incorrect track"))?;
                    bytes_read += 1;
                    buf_4_bytes[i as usize] = buf_1_byte[0];
                }

                let tempo = 60_000_000.0 / u32::from_be_bytes(buf_4_bytes) as f32;
                Ok((bytes_read, MetaEvent::TempoSetting(tempo)))
            },
            0x54 => {
                // SMPTE offset
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::SmpteOffset))
            },
            0x58 => {
                // time signature
                reader.read_exact(&mut buf_1_byte).or(Err("Incorrect track"))?;
                bytes_read += 1;
                let [denominator] = buf_1_byte;
                reader.read_exact(&mut buf_1_byte).or(Err("Incorrect track"))?;
                bytes_read += 1;
                let numerator = (1.0 / 2f32.powf(-(buf_1_byte[0] as f32))) as u8;

                // midi clocks per 4th note, always 0x18
                reader.read_exact(&mut buf_1_byte).or(Err("Incorrect track"))?;
                bytes_read += 1;
                
                // number of 32nd notes in a 4th note, always 0x08
                reader.read_exact(&mut buf_1_byte).or(Err("Incorrect track"))?;
                bytes_read += 1;

                Ok((bytes_read, MetaEvent::TimeSignature((numerator, denominator))))
            },
            0x59 => {
                // key signature
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::KeySignature))
            },
            0x7F => {
                // sequencer specific event
                // unimplemented
                for _ in 0..event_length {
                    reader.read_exact(&mut buf_1_byte).or(Err("Invalid meta event"))?;
                    bytes_read += 1;
                }

                Ok((bytes_read, MetaEvent::SequencerSpecificEvent))
            },
            _ => return Err("Invalid meta event")
        }
    }
}

#[derive(Debug)]
enum MidiEvent {
    ChannelPressure { value: u8 },
    ControlChange { controller: u8, value: u8 },
    NoteOff { key: u8, velocity: u8 },
    NoteOn { key: u8, velocity: u8 },
    PitchWheelChange { value: f32 },
    PolyphonicKeyPressure { key: u8, velocity: u8 },
    ProgramChange { number: u8 },
    Other
}

impl MidiEvent {
    fn from_reader(reader: &mut impl Read, midi_status: u8) -> Result<(u32, Self), &'static str> {
        let mut buf_1_byte = [0u8; 1];
        let mut buf_2_bytes = [0u8; 2];

        let (midi_type, channel) = (midi_status >> 4, midi_status & 0x10);
        match midi_type {
            0b1000 => {
                reader.read_exact(&mut buf_2_bytes).or(Err("Incorrect midi event"))?;
                let [key, velocity] = buf_2_bytes;
                Ok((2, MidiEvent::NoteOff { key, velocity }))                
            },
            0b1001 => {
                reader.read_exact(&mut buf_2_bytes).or(Err("Incorrect midi event"))?;
                let [key, velocity] = buf_2_bytes;
                Ok((2, MidiEvent::NoteOn { key, velocity }))                
            },
            0b1010 => {
                reader.read_exact(&mut buf_2_bytes).or(Err("Incorrect midi event"))?;
                let [key, velocity] = buf_2_bytes;
                Ok((2, MidiEvent::PolyphonicKeyPressure { key, velocity }))                
            },
            0b1011 => {
                reader.read_exact(&mut buf_2_bytes).or(Err("Incorrect midi event"))?;
                let [controller, value] = buf_2_bytes;
                Ok((2, MidiEvent::ControlChange { controller, value }))                
            },
            0b1100 => {
                reader.read_exact(&mut buf_1_byte).or(Err("Incorrect midi event"))?;
                let [number] = buf_1_byte;
                Ok((1, MidiEvent::ProgramChange { number }))                
            },
            0b1101 => {
                reader.read_exact(&mut buf_1_byte).or(Err("Incorrect midi event"))?;
                let [value] = buf_1_byte;
                Ok((1, MidiEvent::ChannelPressure { value }))                
            },
            0b1110 => {
                // untested
                reader.read_exact(&mut buf_2_bytes).or(Err("Incorrect midi event"))?;
                let [least_significant, most_significant] = buf_2_bytes;
                let centre = 8192.0; // 0x2000 as integer, half of what 14 bits can represent
                let value = ((u16::from_be_bytes([most_significant & 0x7F, least_significant & 0x7F]) as f32) - centre) / 16384.0;
                Ok((2, MidiEvent::PitchWheelChange { value }))                
            },
            v => {
                println!("midi status {v} not yet implemented!");
                Err("not implemented")
            }
        }
    }
}

impl Event {
    fn from_reader(reader: &mut impl Read) -> Result<(u32, Self), &'static str > {
        let mut buf_1_byte = [0u8; 1];
        let mut bytes_read = 0;
        // read midi, meta or sysex
        reader.read_exact(&mut buf_1_byte).or(Err("Incorrect event"))?;
        bytes_read += 1;
        let [event_first_byte] = buf_1_byte;
        match event_first_byte {
            0xFF => {
                // meta
                let (event_bytes_read, meta_event) = MetaEvent::from_reader(reader)?;
                bytes_read += event_bytes_read;
                Ok((bytes_read, Event::Meta(meta_event)))
            },
            0xF0 => {
                // sysex form 1
                Ok((bytes_read, Event::Sysex))
            },
            0xF7 => {
                // sysex form 2
                Ok((bytes_read, Event::Sysex))
            }
            midi_status => {
                // midi
                let (event_bytes_read, midi_event) = MidiEvent::from_reader(reader, midi_status)?;
                bytes_read += event_bytes_read;
                Ok((bytes_read, Event::Midi(midi_event)))
            }
        }
    }
}

impl Midi {
    fn from_reader(reader: &mut impl Read) -> Result<Self, &'static str> {
        let header = Header::from_reader(reader)?;
        let tracks = Tracks::from_reader(reader, header.num_tracks as usize)?;       

        Ok(Self{ header, tracks })
    }

    fn to_simple_midi_tracks(&self) -> Vec<SimpleMidiTrack> {
        let division = self.header.division;

        self
            .tracks
            .0
            .iter()
            .map(|track| {
                let mut events = vec![];
                let mut time_in_beats = 0.0;
                let mut time_signature = None;
                let mut tempo = None;
                
                for TrackEvent { v_time, event } in &track.track_events {
                    time_in_beats += *v_time as f32 / division as f32;

                    if let &Event::Meta(MetaEvent::TempoSetting(t)) = event {
                        tempo = Some(t);
                        continue;
                    }

                    if let &Event::Meta(MetaEvent::TimeSignature((numerator, denominator))) = event {
                        time_signature = Some(SimpleMidiTimeSignature::new(numerator, denominator))
                    }

                    let Event::Midi(midi_event) = event else {
                        continue;
                    };

                    let (event_type, key, velocity) = match midi_event {
                        MidiEvent::NoteOff { key, velocity } => ("off", key, velocity),
                        MidiEvent::NoteOn { key, velocity } => ("on", key, velocity),
                        _ => continue
                    };

                    events.push(SimpleMidiEvent::new(time_in_beats, event_type.into(), *key, *velocity));
                }

                SimpleMidiTrack::new(tempo, time_in_beats.ceil() as u32, time_signature, events)
            })
            .collect()
    }
}

fn read_variable_length_value(read: &mut impl Read) -> Result<(u32, u32), std::io::Error> {
    let mut buf_1_byte = [0u8; 1];
    let mut variable_length_bytes = vec![];
    loop {
        read.read_exact(&mut buf_1_byte)?;
        let [byte] = buf_1_byte;
        variable_length_bytes.push(byte & 0x7F);

        if byte & 0x80 == 0 {
            // final byte
            break;
        }             
    }

    let value = 
        variable_length_bytes
            .iter()
            .rev()
            .enumerate()
            .fold(0, |acc, (index, &byte)| acc + 2u32.pow((index as u32) * 7) * byte as u32);
    
    Ok((variable_length_bytes.len() as u32, value))
}