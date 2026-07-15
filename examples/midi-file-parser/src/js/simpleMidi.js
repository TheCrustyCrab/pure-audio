export class SimpleMidiEvent {
    constructor(time, type, key, velocity) {
        this.time = time;
        this.type = type;
        this.key = key;
        this.velocity = velocity;
    }
}

export class SimpleMidiTimeSignature {
    constructor(numerator, denominator) {
        this.numerator = numerator;
        this.denominator = denominator;
    }
}

export class SimpleMidiTrack {
    constructor(tempo, beats, timeSignature, events) {
        this.tempo = tempo;
        this.beats = beats;
        this.timeSignature = timeSignature;
        this.events = events;
    }
}