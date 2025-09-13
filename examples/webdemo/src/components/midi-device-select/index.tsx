import { useCallback, useEffect, useRef, useState } from "react";
import useEventBus from "../../hooks/useEventBus";
import { faSliders } from "@fortawesome/free-solid-svg-icons";
import MenuBarFaIcon from "../menu-bar-fa-icon";

enum MidiStatus {
    NoteOff = 0b1000,
    NoteOn = 0b1001
}

function MidiDeviceSelect() {
    const midiAccess = useRef<MIDIAccess>(null);
    const midiInputs = useRef<MIDIInput[]>([]);
    const [midiInputNames, setMidiInputNames] = useState<string[]>([]);
    const [selectedMidiInputIndex, setSelectedMidiInputIndex] = useState<number>();
    const eventBus = useEventBus();

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
        const data = [...midiMessage.data!.values()];
        if (data.length === 3) {
            const [status, data1, data2] = data;
            if (status >> 4 === MidiStatus.NoteOff) {
                const key = data1;
                const velocity = data2;
                eventBus.publish("midiNoteOff", { key, velocity });
                eventBus.publish("noteOff", { key, velocity });
            } else if (status >> 4 === MidiStatus.NoteOn) {
                const key = data1;
                const velocity = data2;
                eventBus.publish("midiNoteOn", { key, velocity });
                eventBus.publish("noteOn", { key, velocity });
            }
        }
    }

    return (
        <>
            <MenuBarFaIcon icon={faSliders} label="device" />
            <select name="midiDevice" onChange={(evt) => handleSelectMidiInput(parseInt(evt.target.value))}>
                {
                    midiInputNames.map((name, i) => <option key={i} value={i}>{name}</option>)
                }
            </select>
        </>
    );
}

export default MidiDeviceSelect;