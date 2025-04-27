import { ChangeEvent, useRef, useState } from 'react'
import './App.css'
import Keyboard from './components/keyboard'
import { PureAudioWorkletNode } from './assets/oscillator/oscillator' // todo: make generally available, not per audio module

type SynthType = "Oscillator" | "SurgeSynthSaw";

function App() {
    const audioContext = useRef<AudioContext>(null);
    const audioNode = useRef<PureAudioWorkletNode>(null);
    const [activeSynth, setActiveSynth] = useState<SynthType>("Oscillator");

    const synthModules = {
        "Oscillator": { 
            importEsmodule: () => import("./assets/oscillator/oscillator"), 
            importWasm: () => import("./assets/oscillator/oscillator_bg.wasm?url") 
        },
        "SurgeSynthSaw": { 
            importEsmodule: () => import("./assets/surge-synth-saw-demo/surge_synth_saw_demo"), 
            importWasm: () => import("./assets/surge-synth-saw-demo/surge_synth_saw_demo_bg.wasm?url") 
        },
    };

    const initAudio = async () => {
        audioContext.current = new AudioContext();
        await loadSynth(activeSynth);
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

    const loadSynth = async (synthModule: SynthType) => {
        audioNode.current?.disconnect();
        const { importEsmodule, importWasm } = synthModules[synthModule];
        const {
            default: init,
            createAudioNodeWithGeneratedParameterUI
        } = await importEsmodule();
        const { default: url } = await importWasm();
        await init({ module_or_path: url });
        audioNode.current = await createAudioNodeWithGeneratedParameterUI(audioContext.current!, "parameters");
        audioNode.current.addOutputEventListener(console.log);
        audioNode.current!.connect(audioContext.current!.destination);
    }

    return (
        <>
            <button onClick={initAudio}>Init audio</button>
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
            <Keyboard minOctave={2} octaveCount={5} onNoteOff={handleNoteOff} onNoteOn={handleNoteOn} />
        </>
    )
}

export default App
