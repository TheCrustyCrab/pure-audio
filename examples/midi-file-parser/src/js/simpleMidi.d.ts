export interface SimpleMidiEvent {
    time: number,
    type: "off" | "on",
    key: number,
    velocity: number
}

export interface SimpleMidiTimeSignature {
    numerator: number,
    denominator: number
}

export interface SimpleMidiTrack {
    tempo: number,
    beats: number,
    timeSignature?: SimpleMidiTimeSignature,
    events: SimpleMidiEvent[]
}