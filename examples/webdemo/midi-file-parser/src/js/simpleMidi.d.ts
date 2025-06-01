export interface SimpleMidiEvent {
    time: number,
    type: "off" | "on",
    key: number,
    velocity: number
}

export interface SimpleMidiTrack {
    tempo: number,
    beats: number,
    events: SimpleMidiEvent[]
}