import { ChangeEvent, useRef, useState } from "react";
import { PureAudioWorkletNode } from "../../assets/oscillator/oscillator"; // todo: make generally available, not per audio module
import useEventBus from "../../hooks/useEventBus";
import { useAsyncEffect } from "../../hooks/useAsyncEffect";

// the dynamic imports below are crucial to prevent vite from unintentionally removing seemingly unused functions, such as createWasmProcessor, during production build
const synthModules = {
    "Oscillator": {
        importEsmodule: () => import("../../assets/oscillator/oscillator")
    },
    "SurgeSynthSaw": {
        importEsmodule: () => import("../../assets/surge-synth-saw-demo/surge_synth_saw_demo")
    },
};

const effectModules = {
    "Freeverb": {
        importEsmodule: () => import("../../assets/freeverb/freeverb")
    },
    "Tremolo": {
        importEsmodule: () => import("../../assets/tremolo/tremolo")
    }
};

type SynthType = keyof typeof synthModules;
// type EffectType = keyof typeof effectModules;

function Synth({ audioContext }: { audioContext: AudioContext }) {
    const audioNode = useRef<PureAudioWorkletNode>(null);
    const freeverbEffectNode = useRef<PureAudioWorkletNode>(null);
    const tremoloEffectNode = useRef<PureAudioWorkletNode>(null);
    const [activeSynth, setActiveSynth] = useState<SynthType>("Oscillator");
    const eventBus = useEventBus();
    const synthParameterControl = useRef<HTMLDivElement>(null);
    const [freeverbEffectEnabled, setFreeverbEffectEnabled] = useState(false);
    const freeverbParameterControl = useRef<HTMLDivElement>(null);
    const [tremoloEffectEnabled, setTremoloEffectEnabled] = useState(false);
    const tremoloParameterControl = useRef<HTMLDivElement>(null);

    // this hook avoids the 2nd simulatenous initialization in Strict Mode which caused the registerProcessor to fail detecting the first registration
    useAsyncEffect(
        async () => {
            eventBus.subscribe("noteOff", handleNoteOff);
            eventBus.subscribe("noteOn", handleNoteOn);
            eventBus.subscribe("noteScheduleOff", handleNoteScheduleOff);
            eventBus.subscribe("noteScheduleOn", handleNoteScheduleOn);
            
            await loadEffects();
            await loadSynth(activeSynth);
        },
        async () => {
            eventBus.unsubscribe("noteOff", handleNoteOff);
            eventBus.unsubscribe("noteOn", handleNoteOn);
            eventBus.unsubscribe("noteScheduleOff", handleNoteScheduleOff);
            eventBus.unsubscribe("noteScheduleOn", handleNoteScheduleOn);
        },
        [audioContext]
    );

    const handleNoteOn = ({ key, velocity }: { key: number, velocity: number }) => {
        audioNode.current?.noteOn(key, velocity)
    };

    const handleNoteOff = ({ key, velocity }: { key: number, velocity: number }) => {
        audioNode.current?.noteOff(key, velocity)
    };

    const handleNoteScheduleOn = ({ time, key, velocity }: { time: number, key: number, velocity: number }) => {
        audioNode.current?.scheduleNoteOn(time, key, velocity)
    };

    const handleNoteScheduleOff = ({ time, key, velocity }: { time: number, key: number, velocity: number }) => {
        audioNode.current?.scheduleNoteOff(time, key, velocity)
    };
    
    const handleSelectSynthChange = async (evt: ChangeEvent<HTMLSelectElement>) => {
        const synthModule = evt.target.value as SynthType;
        await loadSynth(synthModule);
        setActiveSynth(synthModule);
    };

    const handleFreeverbCheckboxChange = (evt: ChangeEvent<HTMLInputElement>) => {
        const enabled = evt.target.checked;
        setFreeverbEffectEnabled(enabled);
        toggleFreeverbInAudioGraph(enabled);
    }

    const toggleFreeverbInAudioGraph = (enabled: boolean) => {
        if (enabled) {
            audioNode.current?.disconnect();
            audioNode.current?.connect(freeverbEffectNode.current!);
            if (tremoloEffectEnabled) {
                freeverbEffectNode.current?.connect(tremoloEffectNode.current!);
            } else {
                freeverbEffectNode.current?.connect(audioContext.destination);
            }
        } else {
            freeverbEffectNode.current?.disconnect();
            audioNode.current?.disconnect();
            if (tremoloEffectEnabled) {
                audioNode.current?.connect(tremoloEffectNode.current!);
            } else {
                audioNode.current?.connect(audioContext.destination);
            }
        }
    }

    const handleTremoloCheckboxChange = (evt: ChangeEvent<HTMLInputElement>) => {
        const enabled = evt.target.checked;
        setTremoloEffectEnabled(enabled);
        toggleTremoloInAudioGraph(enabled);
    }

    const toggleTremoloInAudioGraph = (enabled: boolean) => {
        if (enabled) {
            if (freeverbEffectEnabled) {
                freeverbEffectNode.current?.disconnect();
                freeverbEffectNode.current?.connect(tremoloEffectNode.current!);
            } else {
                audioNode.current?.disconnect();
                audioNode.current?.connect(tremoloEffectNode.current!);
            }
            tremoloEffectNode.current?.connect(audioContext.destination);
        } else {
            tremoloEffectNode.current?.disconnect();
            if (freeverbEffectEnabled) {
                freeverbEffectNode.current?.disconnect();
                freeverbEffectNode.current?.connect(audioContext.destination);
            } else {
                audioNode.current?.disconnect();
                audioNode.current?.connect(audioContext.destination);
            }
        }
    }

    const handleOutputEvent = (event: any) => {
        if (event.eventType === "scheduleOff") {
            eventBus.publish("noteScheduledOffTriggered", { key: event.key as number, velocity: 0 });
        } else if (event.eventType === "scheduleOn") {
            eventBus.publish("noteScheduledOnTriggered", { key: event.key as number, velocity: 0 });
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
        audioNode.current = await createAudioNodeWithGeneratedParameterUI(audioContext, synthParameterControl.current!);
        audioNode.current.addOutputEventListener(handleOutputEvent);
        toggleFreeverbInAudioGraph(freeverbEffectEnabled);
    }

    const loadEffects = async () => {
        const { importEsmodule: importFreeverbEsmodule } = effectModules["Freeverb"];
        const {
            default: initFreeverb,
            createAudioNodeWithGeneratedParameterUI: createFreeverbAudioNodeWithGeneratedParameterUI
        } = await importFreeverbEsmodule();
        await initFreeverb();
        freeverbEffectNode.current = await createFreeverbAudioNodeWithGeneratedParameterUI(audioContext, freeverbParameterControl.current!);
        const { importEsmodule: importTremoloEsmodule } = effectModules["Tremolo"];
        const {
            default: initTremolo,
            createAudioNodeWithGeneratedParameterUI: createTremoloAudioNodeWithGeneratedParameterUI
        } = await importTremoloEsmodule();
        await initTremolo();
        tremoloEffectNode.current = await createTremoloAudioNodeWithGeneratedParameterUI(audioContext, tremoloParameterControl.current!);
        tremoloEffectNode.current.setHostTempo(120);
    }

    return (
        <>
            <select onChange={handleSelectSynthChange} value={activeSynth}>
                {
                    Object.keys(synthModules).map(synthModule =>
                        <option key={synthModule} value={synthModule}>{synthModule}</option>
                    )
                }
            </select>
            <div ref={synthParameterControl} />
            <hr/>
            <div>
                Enable Freeverb
                <input type="checkbox" checked={freeverbEffectEnabled} onChange={handleFreeverbCheckboxChange} />
            </div>
            <div ref={freeverbParameterControl} style={{ display: freeverbEffectEnabled ? "block" : "none" }} />
            <hr/>
            <div>
                Enable Tremolo
                <input type="checkbox" checked={tremoloEffectEnabled} onChange={handleTremoloCheckboxChange} />
            </div>
            <div ref={tremoloParameterControl} style={{ display: tremoloEffectEnabled ? "block" : "none" }} />
        </>
    );
}

export default Synth;