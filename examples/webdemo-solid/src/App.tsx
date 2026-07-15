import { createSignal, Show, type JSX } from 'solid-js'
import './App.css'
import { Keyboard } from './components/keyboard'
import { EventBus } from './event-bus';
import AudioGraph from './audio-graph';
import styles from "./styles.module.css";
import Synth from './components/synth';
import EffectRack from './components/effect-rack';
import MidiPlayer from './components/midi-player';
import FileDropzone from './components/file-dropzone';
import MenuBarFaIcon from './components/menu-bar-fa-icon';
import { faClock } from '@fortawesome/free-solid-svg-icons';
import MidiDeviceSelect from './components/midi-device-select';

type InitializationState = "Uninitialized" | "Initializing" | "Initialized";

function App() {
    const [initializationState, setInitializationState] = createSignal<InitializationState>("Uninitialized");
    const [tempo, setTempo] = createSignal(130);
    const [midiFile, setMidiFile] = createSignal<{ name: string, data: Uint8Array }>();

    let audioGraph: AudioGraph | undefined;
    const eventBus = new EventBus();

    const initAudio = () => {
        audioGraph = new AudioGraph(new AudioContext());
        setInitializationState("Initializing");
    };

    const handleTempoChange: JSX.EventHandler<HTMLInputElement, Event> = (evt) => {
        const tempo = parseInt(evt.currentTarget.value);
        setTempo(tempo);
        eventBus.publish("hostTempoChange", { tempo });
    };

    const handleFileDrop = async (file: File) => {
        setMidiFile({ name: file.name, data: new Uint8Array(await file.arrayBuffer()) });
    };

    return (
        <div class={styles.container}>
            <Show when={initializationState() !== "Initialized"}>
                <div class={`${styles["preinit-overlay"]} ${initializationState() === "Initializing" ? styles.hiding : ""}`}
                    onClick={initAudio} onAnimationEnd={() => setInitializationState("Initialized")}>
                    <p>Click to start audio</p>
                </div>
            </Show>
            <FileDropzone acceptedTypes={["audio/mid"]} onFileDrop={handleFileDrop}>
                <main class="root">
                    <nav class="menu-bar">
                        <div class="menu-bar-group1">
                            <div class="menu-bar-logo">
                                {/* todo logo: mini audio visualisation? */}
                            </div>
                            pure-audio
                        </div>
                        <div class="menu-bar-group2">
                            <div class="menu-bar-group-item">
                                <MenuBarFaIcon definition={faClock} label="bpm" />
                                <input name="tempo" type="number" class="tempo-input" min={60} max={150} value={130} onChange={handleTempoChange}></input>
                            </div>
                            <Show when={initializationState() !== "Uninitialized"}>
                                <MidiPlayer audioGraph={audioGraph!} eventBus={eventBus} midiFile={midiFile()} tempo={tempo()} />
                            </Show>
                        </div>
                        <div class="menu-bar-group3">
                            <div class="menu-bar-group3-sub">
                                <div class="menu-bar-group-item">
                                    <MidiDeviceSelect eventBus={eventBus} />
                                </div>
                                <div class="menu-bar-group-item">
                                    {/* todo */}
                                    Master volume
                                </div>
                                <div class="menu-bar-group-item">
                                    {/* todo */}
                                    Visualise audio
                                </div>
                            </div>
                        </div>
                    </nav>
                    <div class="workspace-root">
                        {/* this wrapper causes child divs to take the same width as the widest div*/}
                        <div class="workspace-wrapper">
                            <div class="workspace-effects">
                                <Show when={initializationState() !== "Uninitialized"}>
                                    <EffectRack audioGraph={audioGraph!} eventBus={eventBus} tempo={tempo()} />
                                </Show>
                            </div>
                            <div class="workspace-keyboard">
                                <Show when={initializationState() !== "Uninitialized"}>
                                    <Synth audioGraph={audioGraph!} eventBus={eventBus} />
                                </Show>
                                <Keyboard minOctave={2} octaveCount={8} scale={1} eventBus={eventBus} />
                            </div>
                        </div>
                    </div>
                </main>
            </FileDropzone>
        </div>
    )
}

export default App
