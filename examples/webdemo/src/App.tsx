import { ChangeEvent, useState } from "react";
import "./App.css"
import useAudioGraph from "./audio-graph";
import Keyboard from "./components/keyboard";
import MidiDeviceSelect from "./components/midi-device-select";
import Synth from "./components/synth";
import useEventBus from "./hooks/useEventBus";
import FileDropzone from "./components/file-dropzone";
import MidiPlayer from "./components/midi-player";
import EffectRack from "./components/effect-rack";

function App() {
    const [tempo, setTempo] = useState<number>(130);
    const [midiFile, setMidiFile] = useState<{ name: string, data: Uint8Array }>();
    const audioGraph = useAudioGraph();
    const eventBus = useEventBus();

    const handleTempoChange = (evt: ChangeEvent<HTMLInputElement>) => {
        const tempo = parseInt(evt.target.value);
        setTempo(tempo);
        eventBus.publish("hostTempoChange", { tempo });
    }

    const handleFileDrop = async (file: File) => {
        setMidiFile({ name: file.name, data: new Uint8Array(await file.arrayBuffer()) });
    }

    return (
        <FileDropzone acceptedTypes={["audio/mid"]} onFileDrop={handleFileDrop}>
            <main className="root">
                <nav className="menu-bar">
                    <div className="menu-bar-group1">
                        <div className="menu-bar-logo">
                            {/* todo logo: mini audio visualisation? */}
                        </div>
                        pure-audio
                    </div>
                    <div className="menu-bar-group2">
                        <div className="menu-bar-group-item">
                            Tempo <input name="tempo" type="number" className="tempo-input" min={60} max={150} defaultValue={130} onChange={handleTempoChange}></input> bpm
                        </div>
                        {
                            audioGraph === null ? null : <MidiPlayer audioGraph={audioGraph} midiFile={midiFile} tempo={tempo} />
                        }
                    </div>
                    <div className="menu-bar-group3">
                        <div className="menu-bar-group3-sub">
                            <div className="menu-bar-group-item">
                                <MidiDeviceSelect />
                            </div>
                            <div className="menu-bar-group-item">
                                {/* todo */}
                                Master volume
                            </div>
                            <div className="menu-bar-group-item">
                                {/* todo */}
                                Visualise audio
                            </div>
                        </div>
                    </div>
                </nav>
                <div className="workspace-root">
                    {/* this wrapper causes child divs to take the same width as the widest div*/}
                    <div className="workspace-wrapper">
                        <div className="workspace-effects">
                            {
                                audioGraph === null ? null : <EffectRack audioGraph={audioGraph} tempo={tempo} />
                            }
                        </div>
                        <div className="workspace-keyboard">
                            {
                                audioGraph === null ? null : <Synth audioGraph={audioGraph} />
                            }
                            <Keyboard minOctave={2} octaveCount={8} scale={1} />
                        </div>
                    </div>
                </div>
            </main>
        </FileDropzone>
    )
}

export default App