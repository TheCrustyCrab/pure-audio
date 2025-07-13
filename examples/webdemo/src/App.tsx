import { useRef, useState } from 'react'
import './App.css'
import Keyboard from './components/keyboard'
import MidiPlayer from './components/midi-player'
import FileDropzone from './components/file-dropzone'
import Synth from './components/synth'

enum InitializationState {
    Uninitialized,
    Initializing,
    Initialized
}

function App() {
    const audioContext = useRef<AudioContext>(null);
    const [initializationState, setInitializationState] = useState<InitializationState>(InitializationState.Uninitialized);
    const [midiFile, setMidiFile] = useState<{ name: string, data: Uint8Array }>();

    const initAudio = async () => {
        audioContext.current = new AudioContext();
        setInitializationState(InitializationState.Initializing);
    };

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
                    {
                        initializationState === InitializationState.Initialized 
                            ? <Synth audioContext={audioContext.current!} />
                            : null
                    }
                </fieldset>
                <fieldset>
                    <legend>MIDI Player</legend>
                    <MidiPlayer audioContext={audioContext.current} midiFile={midiFile} />
                </fieldset>
                <fieldset>
                    <legend>Keyboard</legend>
                    <Keyboard minOctave={2} octaveCount={5} />
                </fieldset>
            </div>
        </FileDropzone>
    )
}

export default App
