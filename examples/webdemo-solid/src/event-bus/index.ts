type NoteOnEvent = { key: number, velocity: number };
type NoteOffEvent = { key: number, velocity: number };
type NoteScheduleOnEvent = { time: number, key: number, velocity: number };
type NoteScheduleOffEvent = { time: number, key: number, velocity: number };
type HostTempoChangeEvent = { tempo: number };
type HostStartPlayingEvent = void;
type HostStopPlayingEvent = void;

type Event = NoteOnEvent | NoteOffEvent | NoteScheduleOnEvent | NoteScheduleOffEvent | HostTempoChangeEvent | HostStartPlayingEvent | HostStopPlayingEvent;

interface EventTypeMap {
    // events that trigger audio
    "noteOn": NoteOnEvent,
    "noteOff": NoteOffEvent,

    // events that trigger scheduled audio
    "noteScheduleOn": NoteScheduleOnEvent,
    "noteScheduleOff": NoteScheduleOffEvent,

    // events that trigger the keyboard visual
    "midiNoteOn": NoteOnEvent,
    "midiNoteOff": NoteOffEvent,

    // host events
    "hostTempoChange": HostTempoChangeEvent,
    "hostStartPlaying": HostStartPlayingEvent,
    "hostStopPlaying": HostStopPlayingEvent
};


export class EventBus {
    private readonly callbacks: Map<string, Array<(evt: Event) => void>>;

    constructor() {
        this.callbacks = new Map<string, Array<(evt: Event) => void>>([
            ["noteOn", []],
            ["noteOff", []],
            ["noteScheduleOn", []],
            ["noteScheduleOff", []],
            ["midiNoteOn", []],
            ["midiNoteOff", []],
            ["hostTempoChange", []],
            ["hostStartPlaying", []],
            ["hostStopPlaying", []]
        ]);
    }

    publish<K extends keyof EventTypeMap>(type: K, evt: EventTypeMap[K]) {
        this.callbacks.get(type)!.forEach(callback => callback(evt));
    }

    subscribe<K extends keyof EventTypeMap>(type: K, callback: (evt: EventTypeMap[K]) => void) {
        this.callbacks.get(type)!.push(callback as (evt: Event) => void);
    }

    unsubscribe<K extends keyof EventTypeMap>(type: K, callback: (evt: EventTypeMap[K]) => void) {
        const typeCallbacks = this.callbacks.get(type)!;
        const index = typeCallbacks.findIndex(x => x === callback);
        if (index !== -1) {
            typeCallbacks.splice(index, 1);
        }
    }
};