import { createSignal, onCleanup, onMount, type JSX } from "solid-js";
import type AudioGraph from "../../audio-graph";
import type { EventBus } from "../../event-bus";
import type { PureAudioWorkletNode } from "../../assets/oscillator/oscillator";
import styles from "./styles.module.css";

// the dynamic imports below are crucial to prevent vite from unintentionally removing seemingly unused functions, such as createWasmProcessor, during production build
const synthModules = {
    "Oscillator": {
        importEsmodule: () => import("../../assets/oscillator/oscillator")
    },
    "SurgeSynthSaw": {
        importEsmodule: () => import("../../assets/surge-synth-saw-demo/surge_synth_saw_demo")
    },
};

type SynthType = keyof typeof synthModules;

type SynthProps = {
    audioGraph: AudioGraph,
    eventBus: EventBus
}

function Synth(props: SynthProps) {
    const [activeSynth, setActiveSynth] = createSignal<SynthType>("Oscillator");

    let audioNode: PureAudioWorkletNode | undefined;
    let freeverbEffectNode: PureAudioWorkletNode | undefined;
    let tremoloEffectNode: PureAudioWorkletNode | undefined;
    let synthParametersDivRef!: HTMLDivElement;

    onMount(async () => {
        props.eventBus.subscribe("hostTempoChange", handleHostTempoChange);
        props.eventBus.subscribe("hostStartPlaying", handleHostStartPlaying);
        props.eventBus.subscribe("hostStopPlaying", handleHostStopPlaying);
        props.eventBus.subscribe("noteOff", handleNoteOff);
        props.eventBus.subscribe("noteOn", handleNoteOn);
        props.eventBus.subscribe("noteScheduleOff", handleNoteScheduleOff);
        props.eventBus.subscribe("noteScheduleOn", handleNoteScheduleOn);

        await loadSynth(activeSynth());
    });

    onCleanup(() => {
        props.eventBus.unsubscribe("hostTempoChange", handleHostTempoChange);
        props.eventBus.unsubscribe("hostStartPlaying", handleHostStartPlaying);
        props.eventBus.unsubscribe("hostStopPlaying", handleHostStopPlaying);
        props.eventBus.unsubscribe("noteOff", handleNoteOff);
        props.eventBus.unsubscribe("noteOn", handleNoteOn);
        props.eventBus.unsubscribe("noteScheduleOff", handleNoteScheduleOff);
        props.eventBus.unsubscribe("noteScheduleOn", handleNoteScheduleOn);
    });

    const handleHostTempoChange = ({ tempo }: { tempo: number }) => {
        [audioNode, freeverbEffectNode, tremoloEffectNode].forEach(node => node?.setHostTempo(tempo));
    };

    const handleHostStartPlaying = () => {
        [audioNode, freeverbEffectNode, tremoloEffectNode].forEach(node => node?.setHostIsPlaying(true));
    };

    const handleHostStopPlaying = () => {
        [audioNode, freeverbEffectNode, tremoloEffectNode].forEach(node => node?.setHostIsPlaying(false));
    };

    const handleNoteOn = ({ key, velocity }: { key: number, velocity: number }) => {
        audioNode?.noteOn(key, velocity)
    };

    const handleNoteOff = ({ key, velocity }: { key: number, velocity: number }) => {
        audioNode?.noteOff(key, velocity)
    };

    const handleNoteScheduleOn = ({ time, key, velocity }: { time: number, key: number, velocity: number }) => {
        audioNode?.scheduleNoteOn(time, key, velocity)
    };

    const handleNoteScheduleOff = ({ time, key, velocity }: { time: number, key: number, velocity: number }) => {
        audioNode?.scheduleNoteOff(time, key, velocity)
    };

    const handleSelectSynthChange: JSX.EventHandler<HTMLSelectElement, Event> = async (evt) => {
        const synthModule = evt.currentTarget.value as SynthType;
        await loadSynth(synthModule);
        setActiveSynth(synthModule);
    };

    const handleOutputEvent = (event: any) => {
        if (event.eventType === "scheduleOff") {
            props.eventBus.publish("midiNoteOff", { key: event.key as number, velocity: 0 });
        } else if (event.eventType === "scheduleOn") {
            props.eventBus.publish("midiNoteOn", { key: event.key as number, velocity: 0 });
        }
    };

    const loadSynth = async (synthModule: SynthType) => {
        if (audioNode) {
            audioNode.disconnect();
            audioNode.requestStop();
        }
        const { importEsmodule } = synthModules[synthModule];
        const {
            default: init,
            createAudioNodeWithGeneratedParameterUI,
        } = await importEsmodule();
        await init();

        audioNode = await createAudioNodeWithGeneratedParameterUI(props.audioGraph.audioContext, synthParametersDivRef);
        audioNode.addOutputEventListener(handleOutputEvent);
        props.audioGraph.setSynth(audioNode);        
    };

    return (
        <div class={styles.synth}>
            <div class={styles["synth-select"]}>
                Synth:
                <select name="synth" onChange={handleSelectSynthChange} value={activeSynth()}>
                    {
                        Object.keys(synthModules).map(synthModule =>
                            <option value={synthModule}>{synthModule}</option>
                        )
                    }
                </select>
            </div>
            <div class={styles["synth-parameters"]} ref={synthParametersDivRef} />
        </div>
    );
}

export default Synth;