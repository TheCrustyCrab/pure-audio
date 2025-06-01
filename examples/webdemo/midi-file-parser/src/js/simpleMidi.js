export class SimpleMidiEvent {
    constructor(time, type, key, velocity) {
        this.time = time;
        this.type = type;
        this.key = key;
        this.velocity = velocity;
    }
}

export class SimpleMidiTrack {
    constructor(tempo, beats, events) {
        this.tempo = tempo;
        this.beats = beats;
        this.events = events;
    }
}