import { ChangeEvent, useRef, useState } from "react";
import { PureAudioWorkletNode } from "../../assets/oscillator/oscillator"; // todo: make generally available, not per audio module
import useEventBus from "../../hooks/useEventBus";
import { useAsyncEffect } from "../../hooks/useAsyncEffect";

type SynthType = "Oscillator" | "SurgeSynthSaw";

function Synth({ audioContext }: { audioContext: AudioContext }) {
    const audioNode = useRef<PureAudioWorkletNode>(null);
    const [activeSynth, setActiveSynth] = useState<SynthType>("Oscillator");
    const eventBus = useEventBus();
    const synthParameterControl = useRef<HTMLDivElement>(null);

    // this hook avoids the 2nd simulatenous initialization in Strict Mode which caused the registerProcessor to fail detecting the first registration
    useAsyncEffect(
        async () => {
            eventBus.subscribe("noteOff", handleNoteOff);
            eventBus.subscribe("noteOn", handleNoteOn);
            eventBus.subscribe("noteScheduleOff", handleNoteScheduleOff);
            eventBus.subscribe("noteScheduleOn", handleNoteScheduleOn);
            
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

    const synthModules = {
        "Oscillator": {
            importEsmodule: () => import("../../assets/oscillator/oscillator")
        },
        "SurgeSynthSaw": {
            importEsmodule: () => import("../../assets/surge-synth-saw-demo/surge_synth_saw_demo")
        },
    };

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
        audioNode.current!.connect(audioContext.destination);
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
        </>
    );
}

export default Synth;