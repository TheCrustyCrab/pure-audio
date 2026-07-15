import { createSignal, For, onCleanup, onMount } from "solid-js";
import type { PureAudioWorkletNode } from "../../assets/oscillator/oscillator";
import type AudioGraph from "../../audio-graph";
import type { EventBus } from "../../event-bus";
import { createStore, produce } from "solid-js/store";
import FaIcon from "../fa-icon";
import { faMinus, faPlus } from "@fortawesome/free-solid-svg-icons";

const effectModules = {
    "Freeverb": {
        importEsmodule: () => import("../../assets/freeverb/freeverb"),
        configureNode: (_node: PureAudioWorkletNode, _tempo: number) => { }
    },
    "Gain": {
        importEsmodule: () => import("../../assets/gain/gain"),
        configureNode: (_node: PureAudioWorkletNode, _tempo: number) => { }
    },
    "Pan": {
        importEsmodule: () => import("../../assets/pan/pan"),
        configureNode: (_node: PureAudioWorkletNode, _tempo: number) => { }
    },
    "Tremolo": {
        importEsmodule: () => import("../../assets/tremolo/tremolo"),
        configureNode: (node: PureAudioWorkletNode, tempo: number) => node.setHostTempo(tempo)
    }
};

type EffectType = keyof typeof effectModules;

interface EffectEntry {
    type: EffectType,
    enabled: boolean
};

interface EffectNodeControl {
    audioNode: PureAudioWorkletNode | null,
    control: HTMLDivElement | null,
    isLoading: boolean
};

type EffectRackProps = {
    audioGraph: AudioGraph,
    eventBus: EventBus, 
    tempo: number
};

function EffectRack(props: EffectRackProps) {
    const [addEffectEntryType, setAddEffectEntryType] = createSignal<EffectType>("Freeverb");
    const [effectEntries, setEffectEntries] = createStore<EffectEntry[]>([]);
    const effectNodeControls: EffectNodeControl[] = [];

    onMount(() => {
        props.eventBus.subscribe("hostStartPlaying", handleHostStartPlaying);
        props.eventBus.subscribe("hostStopPlaying", handleHostStopPlaying);
        props.eventBus.subscribe("hostTempoChange", handleHostTempoChange);
    });

    onCleanup(() => {
        props.eventBus.unsubscribe("hostStartPlaying", handleHostStartPlaying);
        props.eventBus.unsubscribe("hostStopPlaying", handleHostStopPlaying);
        props.eventBus.unsubscribe("hostTempoChange", handleHostTempoChange);
    });

    const handleHostTempoChange = ({ tempo }: { tempo: number }) => {
        if (isNaN(tempo)) {
            return;
        }

        effectNodeControls.map(effectNodeControl => effectNodeControl.audioNode).forEach(node => node?.setHostTempo(tempo));
    };

    const handleHostStartPlaying = () => {
        effectNodeControls.map(effectNodeControl => effectNodeControl.audioNode).forEach(node => node?.setHostIsPlaying(true));
    };

    const handleHostStopPlaying = () => {
        effectNodeControls.map(effectNodeControl => effectNodeControl.audioNode).forEach(node => node?.setHostIsPlaying(false));
    };

    const handleAddEffectEntryClick = () => {
        effectNodeControls[effectEntries.length] = { audioNode: null, control: null, isLoading: false };
        setEffectEntries(produce(current => current.push({ enabled: true, type: addEffectEntryType() })));
    };

    const handleRemoveEffectEntryClick = (index: number) => {
        props.audioGraph.removeEffect(effectNodeControls[index].audioNode!);
        effectNodeControls.splice(index, 1);
        setEffectEntries(produce(current => current.splice(index, 1)));
    };

    const initAudioNodeForEntry = async (type: EffectType, effectNodeControl: EffectNodeControl, control: HTMLDivElement) => {
        effectNodeControl.isLoading = true;
        const { importEsmodule, configureNode } = effectModules[type];
        const {
            default: init,
            createAudioNodeWithGeneratedParameterUI
        } = await importEsmodule();
        await init();
        const audioNode = await createAudioNodeWithGeneratedParameterUI(props.audioGraph.audioContext, control);
        configureNode(audioNode, props.tempo);
        effectNodeControl.audioNode = audioNode;
        props.audioGraph.appendEffect(audioNode);
    };

    return (
        <>
            <div>
                Add effect:
                <select name="effectType" value={addEffectEntryType()} onChange={evt => setAddEffectEntryType(evt.target.value as EffectType)}>
                    {
                        Object.keys(effectModules).map(key =>
                            <option value={key}>{key}</option>
                        )
                    }
                </select>
                <button onClick={handleAddEffectEntryClick}><FaIcon definition={faPlus} /></button>
            </div>
            <div>
                <hr />
            </div>
            <div class="workspace-effects-nodes">
                <For each={effectEntries}>{(effectEntry, index) =>
                    <div class="workspace-effects-nodes-item">
                        <div class="workspace-effects-nodes-item-header">
                            {effectEntry.type}
                            <button onClick={() => handleRemoveEffectEntryClick(index())}><FaIcon definition={faMinus} /></button>
                        </div>
                        <hr />
                        <div ref={el => {
                            const effectNodeControl = effectNodeControls[(index())];
                            effectNodeControl.control = el;
                            if (!effectNodeControl.isLoading) {
                                initAudioNodeForEntry(effectEntry.type, effectNodeControl, el);
                            }
                        }}></div>
                    </div>
                }</For>
            </div>
        </>
    );
}

export default EffectRack;