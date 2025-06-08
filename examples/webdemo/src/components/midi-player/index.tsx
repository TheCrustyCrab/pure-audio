import { ChangeEvent, useMemo, useRef, useState } from "react";
import { default as initMidiFileParser, midiToSimpleTracks, SimpleMidiEvent, SimpleMidiTrack } from "../../assets/midi-file-parser/midi_file_parser";
import useWasm from "../../hooks/useWasm";
import useInterval from "../../hooks/useInterval";

function MidiPlayer({ audioContext, midiFile, onSchedule }: { audioContext?: AudioContext | null, midiFile: { name: string, data: Uint8Array } | undefined, onSchedule: (event: SimpleMidiEvent) => void }) {
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
        catch(ex) {
            error = ex as string;
        }

        return { midiTracks, error };
    }, [midiFile]);
    const [playMidiStartTime, setPlayMidiStartTime] = useState<number | null>(null);
    const eventIterator = useRef<ArrayIterator<SimpleMidiEvent>>(null);
    const scheduledNotes = useRef<Set<number>>(new Set<number>());
    const [pausingTime, setPausingTime] = useState<number | null>(null);
    const nextEvent = useRef<SimpleMidiEvent>(null);
    const [tempo, setTempo] = useState<number>(130);
    const beatsPerSecond = useMemo(() => tempo / 60, [tempo]);
    const secondsPerBeat = useMemo(() => 60 / tempo, [tempo]);
    const [elapsedTimeInBeats, setElapsedTimeInBeats] = useState(0);
    const interval = 25;
    const scheduleAheadTime = 0.1;

    const midiFileParserLoaded = useWasm(initMidiFileParser);

    const handleMidiTrackChange = (evt: ChangeEvent<HTMLSelectElement>) => {
        stopMidi();
        setSelectedMidiTrackIndex(parseInt(evt.target.value));
        eventIterator.current = null;
    }

    const scheduleMidiEvents = () => {
        const elapsedSeconds = audioContext!.currentTime - playMidiStartTime!;
        const elapsedBeats = elapsedSeconds * beatsPerSecond;
        setElapsedTimeInBeats(current => current + interval / 1000 * beatsPerSecond);

        while (nextEvent.current !== null && nextEvent.current.time < audioContext!.currentTime + scheduleAheadTime) {
            if (pausingTime === null || nextEvent.current.type === "off") {
                onSchedule(nextEvent.current);
            }

            if (nextEvent.current.type === "on") {
                scheduledNotes.current.add(nextEvent.current.key);
            } else {
                scheduledNotes.current.delete(nextEvent.current.key);
                if (pausingTime !== null && scheduledNotes.current.size === 0) {
                    console.log("end pause");
                    setPlayMidiStartTime(null);
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
                eventIterator.current = null;
                break;
            }
        }
    };

    useInterval(scheduleMidiEvents, playMidiStartTime === null ? null : interval);

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
            eventIterator.current = loadedMidiTracks.midiTracks[selectedMidiTrackIndex!].events![Symbol.iterator]();
        }

        if (!advanceNextEvent()) {
            return;
        }

        setPlayMidiStartTime(audioContext!.currentTime);
    }

    const pauseMidi = () => {
        setPausingTime(audioContext!.currentTime);
    }

    const stopMidi = () => {
        setPlayMidiStartTime(null);

        scheduledNotes.current.forEach(note => {
            onSchedule({ key: note, time: audioContext!.currentTime, type: "off", velocity: 0 });
        });
        scheduledNotes.current.clear();

        setElapsedTimeInBeats(0);
        eventIterator.current = null;
    }

    if (!midiFileParserLoaded) {
        return null;
    }

    return (
        <div>
            <p>
            { 
                midiFile 
                    ? loadedMidiTracks.error
                        ? `Failed to load ${midiFile.name}: ${loadedMidiTracks.error}`
                        : `Loaded: ${midiFile.name}`
                    : "Drag and drop a midi file for playback"
            }
            </p>
            { 
                !midiFile || loadedMidiTracks.error 
                    ? null
                    : <>
                        <select onChange={handleMidiTrackChange}>
                            {
                                loadedMidiTracks.midiTracks.map((track, index) => 
                                    <option key={index} value={index}>Track {index}: {track.beats} beats</option>
                                )
                            }
                        </select>
                        <button onClick={playMidiStartTime === null ? playMidi : pauseMidi}>{ playMidiStartTime === null ? "Play" : "Pause" }</button>
                        <button onClick={stopMidi}>Stop</button>
                        <span>{Math.ceil(elapsedTimeInBeats)}</span>
                        <p>
                            Tempo <input type="number" min={60} max={150} defaultValue={130} onChange={evt => setTempo(parseInt(evt.target.value))}></input>
                        </p>
                    </>
            }            
        </div>
    )
}

export default MidiPlayer;

