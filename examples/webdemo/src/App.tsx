import { ChangeEvent, useRef, useState } from 'react'
import './App.css'
import Keyboard from './components/keyboard'
import { PureAudioWorkletNode } from './assets/oscillator/oscillator' // todo: make generally available, not per audio module

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
        audioNode.current.addOutputEventListener(console.log);
        audioNode.current!.connect(audioContext.current!.destination);
    }

    return (
        <div className="container">
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
            <Keyboard minOctave={2} octaveCount={5} onNoteOff={handleNoteOff} onNoteOn={handleNoteOn} />
        </div>
    )
}

export default App
