import { faSliders } from "@fortawesome/free-solid-svg-icons";
import MenuBarFaIcon from "../menu-bar-fa-icon";
import { createSignal, onCleanup, onMount } from "solid-js";
import type { EventBus } from "../../event-bus";

const MidiStatus = {
    NoteOff: 0b1000,
    NoteOn: 0b1001
};

type MidiDeviceSelectProps = {
    eventBus: EventBus
};

function MidiDeviceSelect(props: MidiDeviceSelectProps) {
    const [midiInputNames, setMidiInputNames] = createSignal<string[]>([]);
    const [selectedMidiInputIndex, setSelectedMidiInputIndex] = createSignal<number>();
    let midiAccess: MIDIAccess | undefined;
    let midiInputs: MIDIInput[] = [];

    onMount(async () => {
        // initial activation
        midiAccess = await navigator.requestMIDIAccess();
        midiAccess.addEventListener("statechange", handleMidiStateChange);
        handleMidiStateChange.call(midiAccess);
    });

    onCleanup(() => {
        midiAccess?.removeEventListener("statechange", handleMidiStateChange);
    });

    const handleMidiStateChange = function (this: MIDIAccess) {
        const inputs = [...this.inputs.values()];
        if (inputs.length > 0) {
            if (selectedMidiInputIndex() === undefined) {
                selectMidiInput(inputs[0]);
            }
            midiInputs = inputs;
        }

        setMidiInputNames(inputs.map(input => input.name || ""));
    };

    const handleSelectMidiInput = (index: number) => {
        if (selectedMidiInputIndex() !== undefined) {
            const oldSelectedInput = midiInputs[selectedMidiInputIndex()!];
            oldSelectedInput.onmidimessage = null;
        }

        const selectedInput = midiInputs[index];
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
                props.eventBus.publish("midiNoteOff", { key, velocity });
                props.eventBus.publish("noteOff", { key, velocity });
            } else if (status >> 4 === MidiStatus.NoteOn) {
                const key = data1;
                const velocity = data2;
                props.eventBus.publish("midiNoteOn", { key, velocity });
                props.eventBus.publish("noteOn", { key, velocity });
            }
        }
    };

    return (
        <>
            <MenuBarFaIcon definition={faSliders} label="device" />
            <select name="midiDevice" onChange={(evt) => handleSelectMidiInput(parseInt(evt.target.value))}>
                {
                    midiInputNames().map((name, i) => <option value={i} selected={i === selectedMidiInputIndex()}>{name}</option>)
                }
            </select>
        </>
    );
}

export default MidiDeviceSelect;