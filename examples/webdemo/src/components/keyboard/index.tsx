import { Fragment, useCallback, useEffect, useRef, useState } from 'react';
import { Octave } from './octave';
import useEventBus from '../../hooks/useEventBus';

enum MidiStatus {
    NoteOff = 0b1000,
    NoteOn = 0b1001
}

interface KeyboardProps {
    minOctave: number,
    octaveCount: number
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

function Keyboard({ minOctave, octaveCount }: KeyboardProps) {
    const midiAccess = useRef<MIDIAccess>(null);
    const midiInputs = useRef<MIDIInput[]>([]);
    const [midiInputNames, setMidiInputNames] = useState<string[]>([]);
    const [selectedMidiInputIndex, setSelectedMidiInputIndex] = useState<number>();
    const [pointerChordMode, setPointerChordMode] = useState<PointerChordMode>(PointerChordMode.Note);
    const pointerActiveKey = useRef<number>(null);
    const [activeNotes, setActiveNotes] = useState<number[]>([]);
    const eventBus = useEventBus();

    useEffect(() => {
        // initial activation
        const initMidi = async () => {
            midiAccess.current = await navigator.requestMIDIAccess();
            midiAccess.current.addEventListener("statechange", handleMidiStateChange);
            handleMidiStateChange.call(midiAccess.current);
        };

        initMidi();
        eventBus.subscribe("noteScheduledOffTriggered", handleNoteScheduledOffTriggered);
        eventBus.subscribe("noteScheduledOnTriggered", handleNoteScheduledOnTriggered);

        return () => {
            midiAccess.current?.removeEventListener("statechange", handleMidiStateChange);
            eventBus.unsubscribe("noteScheduledOffTriggered", handleNoteScheduledOffTriggered);
            eventBus.unsubscribe("noteScheduledOnTriggered", handleNoteScheduledOnTriggered);
        };
    }, []);

    useEffect(() => {
        document.body.addEventListener("pointerup", handlePointerUpOrCancel);
        document.body.addEventListener("pointercancel", handlePointerUpOrCancel);

        return () => {
            document.body.removeEventListener("pointerup", handlePointerUpOrCancel);
            document.body.removeEventListener("pointercancel", handlePointerUpOrCancel);
        };
    }, [minOctave, octaveCount, activeNotes]);

    const handleMidiStateChange = useCallback(function (this: MIDIAccess) {
        const inputs = [...this.inputs.values()];
        // selectedMidiInputIndex is always undefined due to the overridden 'this'
        setSelectedMidiInputIndex(currentSelectedMidiInputIndex => {
            let newIndex: number | undefined = currentSelectedMidiInputIndex;
            if (inputs.length > 0) {
                if (currentSelectedMidiInputIndex === undefined) {
                    selectMidiInput(inputs[0]);
                }
                midiInputs.current = inputs;
            } else {            
                newIndex = undefined;
            }

            setMidiInputNames(inputs.map(input => input.name || ""));    
            return newIndex;
        });
    }, []);

    const handleSelectMidiInput = (index: number) => {
        if (selectedMidiInputIndex !== undefined) {
            const oldSelectedInput = midiInputs.current[selectedMidiInputIndex];
            oldSelectedInput.onmidimessage = null;
        }
        const selectedInput = midiInputs.current[index];
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
                eventBus.publish("noteOff", { key, velocity });
                console.log(`Midi note off ${key} with velocity ${velocity}`);
            } else if (status >> 4 === MidiStatus.NoteOn) {
                const key = data1;
                const velocity = data2;
                setActiveNotes(current => [...current, key]);
                eventBus.publish("noteOn", { key, velocity });
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
        keys.forEach(key => eventBus.publish("noteOn", { key, velocity: 127 }));
    };

    const handlePointerUpOrCancel = (_evt: PointerEvent) => {
        document.body.removeEventListener("pointermove", handlePointerMove);
        setActiveNotes(currentActiveNotes => {
            currentActiveNotes.forEach(activeNote => {
                eventBus.publish("noteOff", { key: activeNote, velocity: 127 })
            });
            return [];
        });
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
            eventBus.publish("noteOff", { key: oldKey, velocity: 127 })
        });

        const newKeys = keyOffsets.map(offset => newKeyNumber + offset);
        newKeys.forEach(newKey => {
            eventBus.publish("noteOn", { key: newKey, velocity: 127 })
        });
        setActiveNotes(newKeys);
        pointerActiveKey.current = newKeyNumber;
    }, [pointerChordMode]);

    const handleNoteScheduledOffTriggered = ({ key }: { key: number }) => {
        setActiveNotes(current => current.filter(activeNote => activeNote !== key));
    }

    const handleNoteScheduledOnTriggered = ({ key }: { key: number }) => {
        setActiveNotes(current => [...current, key]);
    }

    return (
        <>
            <div>
                MIDI device:
                <select onChange={(evt) => handleSelectMidiInput(parseInt(evt.target.value))}>
                    {
                        midiInputNames.map((name, i) => <option key={i} value={i}>{name}</option>)
                    }
                </select>
            </div>
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