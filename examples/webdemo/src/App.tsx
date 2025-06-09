import { ChangeEvent, useRef, useState } from 'react'
import './App.css'
import Keyboard from './components/keyboard'
import { PureAudioWorkletNode } from './assets/oscillator/oscillator' // todo: make generally available, not per audio module
import MidiPlayer from './components/midi-player'
import { SimpleMidiEvent } from './assets/midi-file-parser/midi_file_parser'
import FileDropzone from './components/file-dropzone'

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

    const handleFileDrop = async (file: File) => {
        setMidiFile({ name: file.name, data: new Uint8Array(await file.arrayBuffer()) });
    }

    return (
        <FileDropzone acceptedTypes={["audio/mid"]} onFileDrop={handleFileDrop}>
            <div className="container">
                {
                    initializationState !== InitializationState.Initialized
                        ? <div className={`preinit-overlay ${initializationState === InitializationState.Initializing ? "hiding" : ""}`}
                            onClick={initAudio} onAnimationEnd={() => setInitializationState(InitializationState.Initialized)}>
                            <p>Click to start audio</p>
                        </div>
                        : null
                }
                <fieldset>
                    <legend>Synth</legend>
                    <select onChange={handleSelectSynthChange} value={activeSynth}>
                        {
                            Object.keys(synthModules).map(synthModule =>
                                <option key={synthModule} value={synthModule}>{synthModule}</option>
                            )
                        }
                    </select>
                    <div id="parameters"></div>
                </fieldset>
                <fieldset>
                    <legend>MIDI Player</legend>
                    <MidiPlayer audioContext={audioContext.current} midiFile={midiFile} onSchedule={handleMidiPlayerSchedule} />
                </fieldset>
                <fieldset>
                    <legend>Keyboard</legend>
                    <Keyboard minOctave={2} octaveCount={5} scheduledActiveNotes={scheduledActiveNotes} onNoteOff={handleNoteOff} onNoteOn={handleNoteOn} />
                </fieldset>
            </div>
        </FileDropzone>
    )
}

export default App
