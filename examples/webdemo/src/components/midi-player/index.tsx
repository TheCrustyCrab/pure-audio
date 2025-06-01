import { useMemo, useRef, useState } from "react";
import { default as initMidiFileParser, midiToSimpleTracks, SimpleMidiEvent, SimpleMidiTrack } from "../../assets/midi-file-parser/midi_file_parser";
import useWasm from "../../hooks/useWasm";

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
    const schedulerIntervalId = useRef<number>(null);
    const playMidiStartTime = useRef<number>(null);
    const eventIterator = useRef<ArrayIterator<SimpleMidiEvent>>(null);
    const nextEvent = useRef<SimpleMidiEvent>(null);
    const [_tempo, setTempo] = useState<number>(130);
    // retrieve latest tempo value from a running callback, which is not aware of the actual state
    const getTempo = () => {
        let tempo = null;
        setTempo(t => {
            tempo = t;
            return t;
        });
        return tempo!;
    }
    const interval = 25;
    const scheduleAheadTime = 0.1;

    const midiFileParserLoaded = useWasm(initMidiFileParser);

    const scheduleMidiEvents = () => {
        if (nextEvent.current === null) {
            clearInterval(schedulerIntervalId.current!);
            return;
        }

        while (nextEvent.current !== null && nextEvent.current.time < audioContext!.currentTime + scheduleAheadTime) {
            onSchedule(nextEvent.current);

            if (!advanceNextEvent()) {
                clearInterval(schedulerIntervalId.current!);
                break;
            }
        }
    };

    const advanceNextEvent = () => {
        const tempo = getTempo();
        const { value, done } = eventIterator.current!.next();
        if (done) {
            return false;
        }

        const secondsPerBeat = 60 / tempo;
        nextEvent.current = { ...value, time: value.time * secondsPerBeat + playMidiStartTime.current! };
        return true;
    };

    const playMidi = () => {
        playMidiStartTime.current = audioContext!.currentTime;
        eventIterator.current = loadedMidiTracks.midiTracks[selectedMidiTrackIndex!].events![Symbol.iterator]();

        if (!advanceNextEvent()) {
            return;
        }

        schedulerIntervalId.current = setInterval(scheduleMidiEvents, interval);
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
                        <select onChange={evt => setSelectedMidiTrackIndex(parseInt(evt.target.value))}>
                            {
                                loadedMidiTracks.midiTracks.map((track, index) => 
                                    <option key={index} value={index}>Track {index}: {track.beats} beats</option>
                                )
                            }
                        </select>
                        <button onClick={playMidi}>Play</button>
                        <p>
                            Tempo <input type="number" min={60} max={150} defaultValue={130} onChange={evt => setTempo(parseInt(evt.target.value))}></input>
                        </p>
                    </>
            }            
        </div>
    )
}

export default MidiPlayer;

