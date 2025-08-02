import { useState } from 'react'
import './App.css'
import Keyboard from './components/keyboard'
import MidiPlayer from './components/midi-player'
import FileDropzone from './components/file-dropzone'
import Synth from './components/synth'
import EffectRack from './components/effect-rack'
import useAudioGraph from './audio-graph'

function App() {
    const [midiFile, setMidiFile] = useState<{ name: string, data: Uint8Array }>();
    const audioGraph = useAudioGraph();

    const handleFileDrop = async (file: File) => {
        setMidiFile({ name: file.name, data: new Uint8Array(await file.arrayBuffer()) });
    }

    return (
        <FileDropzone acceptedTypes={["audio/mid"]} onFileDrop={handleFileDrop}>
            {
                audioGraph == null
                    ? null
                    : <>
                        <fieldset>
                            <legend>Synth</legend>
                            <Synth audioGraph={audioGraph} />
                        </fieldset>
                        <fieldset>
                            <legend>Effects</legend>
                            <EffectRack audioGraph={audioGraph} />
                        </fieldset>
                        <fieldset>
                            <legend>MIDI Player</legend>
                            <MidiPlayer audioGraph={audioGraph} midiFile={midiFile} />
                        </fieldset>
                    </>
            }
            <fieldset>
                <legend>Keyboard</legend>
                <Keyboard minOctave={2} octaveCount={5} />
            </fieldset>
        </FileDropzone>
    )
}

export default App
