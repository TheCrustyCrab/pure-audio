import { ChangeEvent, useRef, useState } from 'react'
import './App.css'
import Keyboard from './components/keyboard'
import { PureAudioWorkletNode } from './assets/oscillator/oscillator' // todo: make generally available, not per audio module
import MidiPlayer from './components/midi-player'
import { SimpleMidiEvent } from './assets/midi-file-parser/midi_file_parser'

type SynthType = "Oscillator" | "SurgeSynthSaw";

enum InitializationState {
    Uninitialized,
    Initializing,
    Initialized
}

function App() {
    const audioContext = useRef<AudioContext>(null);
    const audioNode = useRef<PureAudioWorkletNode>(null);
    const [initializationState, setInitializationState] = useState<InitializationState>(InitializationState.Uninitialized);
    const [activeSynth, setActiveSynth] = useState<SynthType>("Oscillator");
    const [scheduledActiveNotes, setScheduledActiveNotes] = useState<number[]>([]);
    const [midiFile, setMidiFile] = useState<{ name: string, data: Uint8Array }>();

    const synthModules = {
        "Oscillator": {
            importEsmodule: () => import("./assets/oscillator/oscillator")
        },
        "SurgeSynthSaw": {
            importEsmodule: () => import("./assets/surge-synth-saw-demo/surge_synth_saw_demo")
        },
    };

    const initAudio = async () => {
        audioContext.current = new AudioContext();
        await loadSynth(activeSynth);
        setInitializationState(InitializationState.Initializing);
    };

    const handleSelectSynthChange = async (evt: ChangeEvent<HTMLSelectElement>) => {
        const synthModule = evt.target.value as SynthType;
        await loadSynth(synthModule);
        setActiveSynth(synthModule);
    };

    const handleNoteOn = (key: number, velocity: number) => {
        audioNode.current?.noteOn(key, velocity)
    };

    const handleNoteOff = (key: number, velocity: number) => {
        audioNode.current?.noteOff(key, velocity)
    };

    const handleOutputEvent = (event: any) => {
        if (event.eventType === "scheduleOff") {
            setScheduledActiveNotes(current => current.filter(activeNote => activeNote !== event.key));
        } else if (event.eventType === "scheduleOn") {
            setScheduledActiveNotes(current => [...current, event.key]);
        }
    }

    const loadSynth = async (synthModule: SynthType) => {
        if (audioNode.current) {
            audioNode.current.disconnect();
            audioNode.current.requestStop();
        }
        const { importEsmodule } = synthModules[synthModule];
        const {
            default: init,
            createAudioNodeWithGeneratedParameterUI,
        } = await importEsmodule();
        await init();
        audioNode.current = await createAudioNodeWithGeneratedParameterUI(audioContext.current!, "parameters");
        audioNode.current.addOutputEventListener(handleOutputEvent);
        audioNode.current!.connect(audioContext.current!.destination);
    }

    const handleMidiPlayerSchedule = (event: SimpleMidiEvent) => {
        if (event.type === "off") {
            audioNode.current?.scheduleNoteOff(event.time, event.key, event.velocity);
        } else {
            audioNode.current?.scheduleNoteOn(event.time, event.key, event.velocity);
        }
    }
 
    const handleDragEnter = (_evt: React.DragEvent) => {
        // todo
    };

    const handleDragLeave = (_evt: React.DragEvent) => {
        // todo
    };    

    const handleDrop = async (evt: React.DragEvent) => {
        console.log("drop");
        console.log(evt);

        evt.preventDefault();

        if (!evt.dataTransfer) {
            return;
        }

        if (evt.dataTransfer.items) {
            if (evt.dataTransfer.items.length != 1) {
                return;
            }

            const item = evt.dataTransfer.items[0];
            if (item.kind === "file") {
                const file = item.getAsFile()!;
                const name = file.name;
                const data = new Uint8Array(await file.arrayBuffer());
                setMidiFile({ name, data });
            };
        } else {
            if (evt.dataTransfer.files.length != 1) {
                return;
            }

            const file = evt.dataTransfer.files[0];
            const name = file.name;
            const data = new Uint8Array(await file.arrayBuffer());
            setMidiFile({ name, data });
        }
    };

    return (
        <div className="container" onDragOver={evt => evt.preventDefault()} onDragEnter={handleDragEnter} onDragLeave={handleDragLeave} onDrop={handleDrop}>
            {
                initializationState !== InitializationState.Initialized
                    ? <div className={`preinit-overlay ${initializationState === InitializationState.Initializing ? "hiding" : ""}`}
                        onClick={initAudio} onAnimationEnd={() => setInitializationState(InitializationState.Initialized)}>
                        <p>Click to start audio</p>
                    </div>
                    : null
            }
            <div>
                <select onChange={handleSelectSynthChange} value={activeSynth}>
                    {
                        Object.keys(synthModules).map(synthModule =>
                            <option key={synthModule} value={synthModule}>{synthModule}</option>
                        )
                    }
                </select>
            </div>
            <div id="parameters"></div>
            <MidiPlayer audioContext={audioContext.current} midiFile={midiFile} onSchedule={handleMidiPlayerSchedule} />
            <Keyboard minOctave={2} octaveCount={5} scheduledActiveNotes={scheduledActiveNotes} onNoteOff={handleNoteOff} onNoteOn={handleNoteOn} />
        </div>
    )
}

export default App
