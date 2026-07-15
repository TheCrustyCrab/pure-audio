import { createEffect, createSignal, For, onCleanup, onMount, Show, type JSX } from "solid-js";
import { default as initMidiFileParser, midiToSimpleTracks, type SimpleMidiEvent, type SimpleMidiTrack } from "../../assets/midi-file-parser/midi_file_parser";
import type AudioGraph from "../../audio-graph";
import ScheduleWorker from "./scheduleWorker?worker";
import type { EventBus } from "../../event-bus";
import BeatBarIndicator from "../beat-bar-indicator";
import { faFileImport } from "@fortawesome/free-solid-svg-icons/faFileImport";
import MenuBarFaIcon from "../menu-bar-fa-icon";
import FaIcon from "../fa-icon";
import { faPause, faPlay, faStop } from "@fortawesome/free-solid-svg-icons";

// https://web.dev/articles/audio-scheduling
const intervalMillis = 25;
const intervalSeconds = intervalMillis / 1000;
const scheduleAheadTime = 0.1;

type MidiPlayerProps = {
    tempo: number,
    audioGraph: AudioGraph,
    eventBus: EventBus,
    midiFile: { name: string, data: Uint8Array } | undefined
};

// In Solid, reactive properties shouldn't be destructured yet in the function signature
function MidiPlayer(props: MidiPlayerProps) {
    const [selectedMidiTrackIndex, setSelectedMidiTrackIndex] = createSignal<number>();
    const [loadedMidiTracks, setLoadedMidiTracks] = createSignal<{ midiTracks: SimpleMidiTrack[], error: string | null }>({ midiTracks: [], error: null });
    const selectedMidiTrack = () => {
        if (!props.midiFile || loadedMidiTracks().error) {
            return null;
        }

        return loadedMidiTracks().midiTracks[selectedMidiTrackIndex()!];
    };
    const [playMidiStartTime, setPlayMidiStartTime] = createSignal(0);
    const [pausingTime, setPausingTime] = createSignal<number>();
    const beatsPerSecond = () => props.tempo / 60;
    const secondsPerBeat = () => 60 / props.tempo;
    const [elapsedTimeInBeats, setElapsedTimeInBeats] = createSignal(0);
    const elapsedTimeInBeatsFloored = () => Math.floor(elapsedTimeInBeats());
    const [midiFileParserLoaded, setMidiFileParserLoaded] = createSignal(false);

    let eventIterator: ArrayIterator<SimpleMidiEvent> | undefined;
    let scheduledNotes = new Set<number>();
    let nextEvent: SimpleMidiEvent | undefined;
    let scheduleWorker: Worker | undefined;
    
    onMount(async () => {
        await initMidiFileParser();
        setMidiFileParserLoaded(true);
        scheduleWorker = new ScheduleWorker();
        scheduleWorker.onmessage = scheduleMidiEvents;
    });

    onCleanup(() => {
        scheduleWorker!.terminate();
    });
    
    createEffect(() => {
        let midiTracks: SimpleMidiTrack[] = [];
        let error: string | null = null;

        if (props.midiFile) {
            try {
                midiTracks = midiToSimpleTracks(props.midiFile.data);
                if (midiTracks.length === 0) {
                    error = "No midi tracks found"
                } else {
                    setSelectedMidiTrackIndex(0);
                }
            }
            catch (ex) {
                error = ex as string;
            }
        }

        setLoadedMidiTracks({ midiTracks, error });
    });

    const handleMidiTrackChange: JSX.EventHandler<HTMLSelectElement, Event> = evt => {
        stopMidi();
        setSelectedMidiTrackIndex(parseInt(evt.currentTarget.value));
        eventIterator = undefined;
    };

    const startScheduler = () => scheduleWorker!.postMessage({ type: "start", interval: intervalMillis });

    const stopScheduler = () => scheduleWorker!.postMessage({ type: "stop" });

    const scheduleMidiEvents = () => {
        const elapsedSeconds = props.audioGraph.currentTime - playMidiStartTime();
        const bps = beatsPerSecond();
        const elapsedBeats = elapsedSeconds * bps;
        setElapsedTimeInBeats(current => current + intervalSeconds * bps);

        while (nextEvent !== undefined && nextEvent.time < props.audioGraph.currentTime + scheduleAheadTime) {
            if (pausingTime() === undefined || nextEvent.type === "off") {
                const { time, key, velocity } = nextEvent;
                const type = nextEvent.type === "on" ? "noteScheduleOn" : "noteScheduleOff";
                props.eventBus.publish(type, { time, key, velocity });
            }

            if (nextEvent.type === "on") {
                scheduledNotes.add(nextEvent.key);
            } else {
                scheduledNotes.delete(nextEvent.key);
                if (pausingTime() !== undefined && scheduledNotes.size === 0) {
                    setPlayMidiStartTime(0);
                    stopScheduler();
                    setPausingTime(undefined);
                    const remainingEvents = [...eventIterator!].map(event => {
                        return { ...event, time: event.time - elapsedBeats };
                    });
                    eventIterator = remainingEvents[Symbol.iterator]();
                    break;
                }
            }

            if (!advanceNextEvent()) {
                setPlayMidiStartTime(0);
                stopScheduler();
                eventIterator = undefined;
                break;
            }
        }
    };

    const advanceNextEvent = () => {
        const { value, done } = eventIterator!.next();
        if (done) {
            return false;
        }

        nextEvent = { ...value, time: value.time * secondsPerBeat() + playMidiStartTime() };
        return true;
    };

    const playMidi = () => {
        if (eventIterator === undefined) {
            setElapsedTimeInBeats(0);
            eventIterator = selectedMidiTrack()!.events![Symbol.iterator]();
        }

        if (!advanceNextEvent()) {
            return;
        }

        setPlayMidiStartTime(props.audioGraph.currentTime);
        startScheduler();

        props.eventBus.publish("hostStartPlaying", undefined);
    }

    const pauseMidi = () => {
        setPausingTime(props.audioGraph.currentTime);
        props.eventBus.publish("hostStopPlaying", undefined);
    }

    const stopMidi = () => {
        setPlayMidiStartTime(0);
        stopScheduler();

        scheduledNotes.forEach(note => {
            props.eventBus.publish("noteScheduleOff", { time: props.audioGraph.currentTime, key: note, velocity: 0 });
        });
        scheduledNotes.clear();

        setElapsedTimeInBeats(0);
        eventIterator = undefined;
        props.eventBus.publish("hostStopPlaying", undefined);
    }

    return (
        <Show when={midiFileParserLoaded()}>
            <div class="menu-bar-group-item">
                <MenuBarFaIcon definition={faFileImport} label="midi" />
                {
                    props.midiFile
                        ? loadedMidiTracks().error
                            ? `Failed to load ${props.midiFile.name}: ${loadedMidiTracks().error}`
                            : props.midiFile.name
                        : "none"
                }
            </div>
            <Show when={props.midiFile && !loadedMidiTracks().error}>
                <div class="menu-bar-group-item">
                    <select onChange={handleMidiTrackChange}>
                        <For each={loadedMidiTracks().midiTracks}>{(track, index) => {
                            const timeSignature = track.timeSignature ? track.timeSignature.numerator + "/" + track.timeSignature.denominator : null;
                            let trackLabel = `Track ${index()}: ${track.beats} beats`;
                            if (timeSignature)
                                trackLabel += `, ${timeSignature}`;
                            return <option value={index()}>{trackLabel}</option>;
                        }}</For>
                    </select>
                </div>
            </Show>
            <Show when={selectedMidiTrack()?.timeSignature}>
                <div class="menu-bar-group-item">
                    <BeatBarIndicator currentBeat={elapsedTimeInBeatsFloored() % selectedMidiTrack()!.timeSignature!.numerator}
                        beatsPerBar={selectedMidiTrack()!.timeSignature!.numerator} />
                </div>
            </Show>
            <div class="menu-bar-group-item">
                {playMidiStartTime() === 0
                    ? <button onClick={playMidi} disabled={selectedMidiTrack() === null}><FaIcon definition={faPlay} /></button>
                    : <button onClick={pauseMidi} disabled={selectedMidiTrack() === null}><FaIcon definition={faPause} /></button>
                }
                <button onClick={stopMidi} disabled={selectedMidiTrack() === null}><FaIcon definition={faStop} /></button>
            </div>
            <div class="menu-bar-group-item">
                {elapsedTimeInBeatsFloored() + 1}
            </div>
        </Show>
    )
}

export default MidiPlayer;