import { ChangeEvent, useEffect, useMemo, useRef, useState } from "react";
import { default as initMidiFileParser, midiToSimpleTracks, SimpleMidiEvent, SimpleMidiTrack } from "../../assets/midi-file-parser/midi_file_parser";
import useWasm from "../../hooks/useWasm";
import useEventBus from "../../hooks/useEventBus";
import { AudioGraph } from "../../audio-graph";
import BeatBarIndicator from "../beat-bar-indicator";
import ScheduleWorker from "./scheduleWorker?worker";

// https://web.dev/articles/audio-scheduling
const intervalMillis = 25;
const intervalSeconds = intervalMillis / 1000;
const scheduleAheadTime = 0.1;

interface MidiPlayerProps {
    tempo: number,
    audioGraph: AudioGraph,
    midiFile: { name: string, data: Uint8Array } | undefined
}

function MidiPlayer({ tempo, audioGraph, midiFile }: MidiPlayerProps) {
    const [selectedMidiTrackIndex, setSelectedMidiTrackIndex] = useState<number>();
    const loadedMidiTracks = useMemo(() => {
        let midiTracks: SimpleMidiTrack[] = [];
        let error: string | null = null;

        if (!midiFile) {
            return { midiTracks, error };
        }

        try {
            midiTracks = midiToSimpleTracks(midiFile.data);
            if (midiTracks.length === 0) {
                error = "No midi tracks found"
            } else {
                setSelectedMidiTrackIndex(0);
            }
        }
        catch (ex) {
            error = ex as string;
        }

        return { midiTracks, error };
    }, [midiFile]);
    const selectedMidiTrack = useMemo(() => {
        if (!midiFile || loadedMidiTracks.error) {
            return null;
        }

        return loadedMidiTracks.midiTracks[selectedMidiTrackIndex!];
    }, [midiFile, loadedMidiTracks, selectedMidiTrackIndex]);

    const [playMidiStartTime, setPlayMidiStartTime] = useState<number | null>(null);
    const eventIterator = useRef<ArrayIterator<SimpleMidiEvent>>(null);
    const scheduledNotes = useRef<Set<number>>(new Set<number>());
    const [pausingTime, setPausingTime] = useState<number | null>(null);
    const nextEvent = useRef<SimpleMidiEvent>(null);
    const beatsPerSecond = useMemo(() => tempo / 60, [tempo]);
    const secondsPerBeat = useMemo(() => 60 / tempo, [tempo]);

    const scheduleWorker = useRef<Worker>(null);
    useEffect(() => {
        scheduleWorker.current = new ScheduleWorker();
        return () => scheduleWorker.current!.terminate();
    }, []);

    useEffect(() => {
        scheduleWorker.current!.onmessage = scheduleMidiEvents;
    }, [playMidiStartTime, pausingTime]);

    const [elapsedTimeInBeats, setElapsedTimeInBeats] = useState(0);
    const elapsedTimeInBeatsFloored = useMemo(() => Math.floor(elapsedTimeInBeats), [elapsedTimeInBeats]);
    const eventBus = useEventBus();

    const midiFileParserLoaded = useWasm(initMidiFileParser);

    const handleMidiTrackChange = (evt: ChangeEvent<HTMLSelectElement>) => {
        stopMidi();
        setSelectedMidiTrackIndex(parseInt(evt.target.value));
        eventIterator.current = null;
    }

    const startScheduler = () => scheduleWorker.current!.postMessage({ type: "start", interval: intervalMillis });

    const stopScheduler = () => scheduleWorker.current!.postMessage({ type: "stop" });

    const scheduleMidiEvents = () => {
        const elapsedSeconds = audioGraph.currentTime - playMidiStartTime!;
        const elapsedBeats = elapsedSeconds * beatsPerSecond;
        setElapsedTimeInBeats(current => current + intervalSeconds * beatsPerSecond);

        while (nextEvent.current !== null && nextEvent.current.time < audioGraph.currentTime + scheduleAheadTime) {
            if (pausingTime === null || nextEvent.current.type === "off") {
                const { time, key, velocity } = nextEvent.current;
                const type = nextEvent.current.type === "on" ? "noteScheduleOn" : "noteScheduleOff";
                eventBus.publish(type, { time, key, velocity });
            }

            if (nextEvent.current.type === "on") {
                scheduledNotes.current.add(nextEvent.current.key);
            } else {
                scheduledNotes.current.delete(nextEvent.current.key);
                if (pausingTime !== null && scheduledNotes.current.size === 0) {
                    setPlayMidiStartTime(null);
                    stopScheduler();
                    setPausingTime(null);
                    const remainingEvents = [...eventIterator.current!].map(event => {
                        return { ...event, time: event.time - elapsedBeats };
                    });
                    eventIterator.current = remainingEvents[Symbol.iterator]();
                    break;
                }
            }

            if (!advanceNextEvent()) {
                setPlayMidiStartTime(null);
                stopScheduler();
                eventIterator.current = null;
                break;
            }
        }
    };

    const advanceNextEvent = () => {
        const { value, done } = eventIterator.current!.next();
        if (done) {
            return false;
        }

        nextEvent.current = { ...value, time: value.time * secondsPerBeat + playMidiStartTime! };
        return true;
    };

    const playMidi = () => {
        if (eventIterator.current === null) {
            setElapsedTimeInBeats(0);
            eventIterator.current = selectedMidiTrack!.events![Symbol.iterator]();
        }

        if (!advanceNextEvent()) {
            return;
        }

        setPlayMidiStartTime(audioGraph.currentTime);
        startScheduler();

        eventBus.publish("hostStartPlaying", undefined);
    }

    const pauseMidi = () => {
        setPausingTime(audioGraph.currentTime);
        eventBus.publish("hostStopPlaying", undefined);
    }

    const stopMidi = () => {
        setPlayMidiStartTime(null);
        stopScheduler();

        scheduledNotes.current.forEach(note => {
            eventBus.publish("noteScheduleOff", { time: audioGraph.currentTime, key: note, velocity: 0 });
        });
        scheduledNotes.current.clear();

        setElapsedTimeInBeats(0);
        eventIterator.current = null;
        eventBus.publish("hostStopPlaying", undefined);
    }

    if (!midiFileParserLoaded) {
        return null;
    }

    return (
        <>
            <div className="menu-bar-group-item">
                MIDI file:
                {
                    midiFile
                        ? loadedMidiTracks.error
                            ? `Failed to load ${midiFile.name}: ${loadedMidiTracks.error}`
                            : midiFile.name
                        : "Drag and drop"
                }
            </div>
            {
                !midiFile || loadedMidiTracks.error
                    ? null
                    : <div className="menu-bar-group-item">
                        <select onChange={handleMidiTrackChange}>
                            {
                                loadedMidiTracks.midiTracks.map((track, index) => {
                                    const timeSignature = track.timeSignature ? track.timeSignature.numerator + "/" + track.timeSignature.denominator : null;
                                    let trackLabel = `Track ${index}: ${track.beats} beats`;
                                    if (timeSignature)
                                        trackLabel += `, ${timeSignature}`;
                                    return <option key={index} value={index}>{trackLabel}</option>;
                                })
                            }
                        </select>
                    </div>
            }
            {
                !selectedMidiTrack || !selectedMidiTrack.timeSignature
                    ? null
                    : <div className="menu-bar-group-item">
                        <BeatBarIndicator currentBeat={elapsedTimeInBeatsFloored % selectedMidiTrack.timeSignature.numerator} 
                            beatsPerBar={selectedMidiTrack.timeSignature.numerator} />
                    </div>
            }
            <div className="menu-bar-group-item">
                {playMidiStartTime === null
                    ? <button onClick={playMidi} disabled={selectedMidiTrack === null}>&#9654;&#65039;</button>
                    : <button onClick={pauseMidi} disabled={selectedMidiTrack === null}>&#9208;&#65039;</button>
                }
                <button onClick={stopMidi} disabled={selectedMidiTrack === null}>&#9209;&#65039;</button>
            </div>
            <div className="menu-bar-group-item">
                {elapsedTimeInBeatsFloored + 1}
            </div>
        </>
    )
}

export default MidiPlayer;

