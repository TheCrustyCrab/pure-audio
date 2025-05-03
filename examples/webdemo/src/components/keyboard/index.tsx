import { Fragment, useCallback, useEffect, useRef, useState } from 'react';
import { Octave } from './octave';

enum MidiStatus {
    NoteOff = 0b1000,
    NoteOn = 0b1001
}

interface KeyboardProps {
    minOctave: number,
    octaveCount: number,
    onNoteOn: (key: number, velocity: number) => void,
    onNoteOff: (key: number, velocity: number) => void
}

enum PointerChordMode {
    Note = "Note",
    Major = "Major",
    Minor = "Minor",
    Sus2 = "Sus2",
    Sus4 = "Sus4"
}

const chordKeyOffsets: { [key in PointerChordMode]: Array<number> } = {
    [PointerChordMode.Note]: [0],
    [PointerChordMode.Major]: [0, 4, 7],
    [PointerChordMode.Minor]: [0, 3, 7],
    [PointerChordMode.Sus2]: [0, 2, 7],
    [PointerChordMode.Sus4]: [0, 5, 7]
}

function Keyboard({ minOctave, octaveCount, onNoteOn, onNoteOff }: KeyboardProps) {
    const midiAccess = useRef<MIDIAccess>(null);
    const [midiInputs, setMidiInputs] = useState<MIDIInput[]>([]);
    const [selectedMidiInputIndex, setSelectedMidiInputIndex] = useState<number>();
    const [pointerChordMode, setPointerChordMode] = useState<PointerChordMode>(PointerChordMode.Note);
    const pointerActiveKey = useRef<number>(null);
    const [activeNotes, setActiveNotes] = useState<number[]>([]);

    useEffect(() => {
        // initial activation
        const initMidi = async () => {
            midiAccess.current = await navigator.requestMIDIAccess();
            midiAccess.current.addEventListener("statechange", handleMidiStateChange);
            handleMidiStateChange.call(midiAccess.current);
        };

        initMidi();

        return () => {
            midiAccess.current?.removeEventListener("statechange", handleMidiStateChange);
        };
    }, []);

    useEffect(() => {
        document.body.addEventListener("pointerup", handlePointerUpOrCancel);
        document.body.addEventListener("pointercancel", handlePointerUpOrCancel);
        if (selectedMidiInputIndex !== undefined) {
            midiInputs[selectedMidiInputIndex].onmidimessage = handleMidiMessage;
        }

        return () => {
            document.body.removeEventListener("pointerup", handlePointerUpOrCancel);
            document.body.removeEventListener("pointercancel", handlePointerUpOrCancel);
        };
    }, [minOctave, octaveCount, activeNotes, onNoteOn, onNoteOff]);

    const handleMidiStateChange = useCallback(function (this: MIDIAccess) {
        const inputs = [...this.inputs.values()];
        if (inputs.length > 0) {
            if (selectedMidiInputIndex === undefined) {
                selectMidiInput(inputs[0]);
                setSelectedMidiInputIndex(0);
            }
        } else {            
            setSelectedMidiInputIndex(undefined);
        }

        setMidiInputs(inputs);
    }, [selectedMidiInputIndex]);

    const handleSelectMidiInput = (index: number) => {
        console.log(index);
        const selectedInput = midiInputs[index];
        selectMidiInput(selectedInput);
        setSelectedMidiInputIndex(index);
    };

    const selectMidiInput = (input: MIDIInput) => {
        input.onmidimessage = handleMidiMessage;
    };

    const handleMidiMessage = (midiMessage: MIDIMessageEvent) => {
        // console.log(midiMessage.data);
        const data = [...midiMessage.data!.values()];
        if (data.length === 3) {
            const [status, data1, data2] = data;
            if (status >> 4 === MidiStatus.NoteOff) {
                const key = data1;
                const velocity = data2;
                setActiveNotes(current => current.filter(activeNote => activeNote !== key));
                onNoteOff(key, velocity);
                console.log(`Midi note off ${key} with velocity ${velocity}`);
            } else if (status >> 4 === MidiStatus.NoteOn) {
                const key = data1;
                const velocity = data2;
                setActiveNotes(current => [...current, key]);
                onNoteOn(key, velocity);
                console.log(`Midi note on ${key} with velocity ${velocity}`);
            }
        }
    }

    const handleNoteOn = (key: number) => {
        document.body.addEventListener("pointermove", handlePointerMove);
        pointerActiveKey.current = key;
        const keyOffsets = chordKeyOffsets[pointerChordMode];
        const keys = keyOffsets.map(offset => key + offset);
        setActiveNotes([...activeNotes, ...keys]);
        keys.forEach(key => {
            onNoteOn(key, 127);
            console.log(`Note on: ${key}`);
        });
    };

    const handlePointerUpOrCancel = (_evt: PointerEvent) => {
        document.body.removeEventListener("pointermove", handlePointerMove);
        activeNotes.forEach(activeNote => {
            onNoteOff(activeNote, 127);
            console.log(`Note off: ${activeNote}`);
        });
        setActiveNotes([]);
    };

    const handlePointerMove = useCallback((evt: PointerEvent) => {
        const newKey = (evt.target as SVGUseElement).getAttribute("data-key");
        if (newKey === null) {
            return;
        }

        const newKeyNumber = parseInt(newKey);
        if (newKeyNumber === pointerActiveKey.current) {
            return;
        }

        const keyOffsets = chordKeyOffsets[pointerChordMode];
        const oldKeys = keyOffsets.map(offset => pointerActiveKey.current! + offset);
        oldKeys.forEach(oldKey => {
            onNoteOff(oldKey, 127);
        });

        const newKeys = keyOffsets.map(offset => newKeyNumber + offset);
        newKeys.forEach(newKey => {
            onNoteOn(newKey, 127);
        });
        setActiveNotes(newKeys);
        pointerActiveKey.current = newKeyNumber;
    }, [pointerChordMode]);

    return (
        <>
            <div>
                {
                    [PointerChordMode.Note, PointerChordMode.Major, PointerChordMode.Minor, PointerChordMode.Sus2, PointerChordMode.Sus4].map(mode =>
                        <Fragment key={mode}>
                            <input type="radio" id={mode} checked={pointerChordMode === mode} onChange={() => setPointerChordMode(mode)} />
                            <label htmlFor={mode}>{mode}</label>
                        </Fragment>
                    )
                }
            </div>
            <div>
                MIDI device:
                <select onChange={(evt) => handleSelectMidiInput(parseInt(evt.target.value))}>
                    {
                        midiInputs.map((input, i) => <option key={i} value={i}>{input.name}</option>)
                    }
                </select>
            </div>
            {
                [...Array(octaveCount)].map((_, i) => {
                    const octaveIndex = i + minOctave;
                    return <Octave key={octaveIndex} index={octaveIndex} onNoteOn={handleNoteOn} activeNotes={activeNotes} />;
                })
            }
        </>
    )
}

export default Keyboard;