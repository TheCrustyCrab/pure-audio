import { useEffect, useRef, useState } from "react";
import { PureAudioWorkletNode } from "../../assets/oscillator/oscillator";
import useEventBus from "../../hooks/useEventBus";
import { AudioGraph } from "../../audio-graph";

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
    id: string, // UUID for correct change detection: effect name and index are not unique
    type: EffectType,
    enabled: boolean
}

interface EffectNodeControl {
    audioNode: PureAudioWorkletNode | null,
    control: HTMLDivElement | null,
    isLoading: boolean
}

function EffectRack({ audioGraph, tempo }: { audioGraph: AudioGraph, tempo: number }) {
    const [addEffectEntryType, setAddEffectEntryType] = useState<EffectType>("Freeverb");
    const effectNodeControls = useRef<EffectNodeControl[]>([]);
    const [effectEntries, setEffectEntries] = useState<EffectEntry[]>([]);
    const eventBus = useEventBus();

    useEffect(() => {
        eventBus.subscribe("hostStartPlaying", handleHostStartPlaying);
        eventBus.subscribe("hostStopPlaying", handleHostStopPlaying);
        eventBus.subscribe("hostTempoChange", handleHostTempoChange);

        return () => {
            eventBus.unsubscribe("hostStartPlaying", handleHostStartPlaying);
            eventBus.unsubscribe("hostStopPlaying", handleHostStopPlaying);
            eventBus.unsubscribe("hostTempoChange", handleHostTempoChange);
        };
    }, []);

    const handleHostTempoChange = ({ tempo }: { tempo: number }) => {
        if (isNaN(tempo)) {
            return;
        }

        effectNodeControls.current.map(effectNodeControl => effectNodeControl.audioNode).forEach(node => node?.setHostTempo(tempo));
    }

    const handleHostStartPlaying = () => {
        effectNodeControls.current.map(effectNodeControl => effectNodeControl.audioNode).forEach(node => node?.setHostIsPlaying(true));
    }

    const handleHostStopPlaying = () => {
        effectNodeControls.current.map(effectNodeControl => effectNodeControl.audioNode).forEach(node => node?.setHostIsPlaying(false));
    }

    const handleAddEffectEntryClick = () => {
        effectNodeControls.current[effectEntries.length] = { audioNode: null, control: null, isLoading: false };
        setEffectEntries(currentEntries => [...currentEntries, { id: crypto.randomUUID(), enabled: true, type: addEffectEntryType }]);
    }

    const handleRemoveEffectEntryClick = (index: number) => {
        audioGraph.removeEffect(effectNodeControls.current[index].audioNode!);
        effectNodeControls.current.splice(index, 1);
        setEffectEntries(currentEffectEntries => {
            const effectsToKeep = currentEffectEntries.filter((_, i) => i !== index);
            return effectsToKeep;
        });
    }

    const initAudioNodeForEntry = async (type: EffectType, effectNodeControl: EffectNodeControl, control: HTMLDivElement) => {
        effectNodeControl.isLoading = true;
        const { importEsmodule, configureNode } = effectModules[type];
        const {
            default: init,
            createAudioNodeWithGeneratedParameterUI
        } = await importEsmodule();
        await init();
        const audioNode = await createAudioNodeWithGeneratedParameterUI(audioGraph.audioContext, control);
        configureNode(audioNode, tempo);
        effectNodeControl.audioNode = audioNode;
        audioGraph.appendEffect(audioNode);
    }

    return (
        <>
            <div>
                Add effect:
                <select name="effectType" value={addEffectEntryType} onChange={evt => setAddEffectEntryType(evt.target.value as EffectType)}>
                    {
                        Object.keys(effectModules).map(key =>
                            <option key={key} value={key}>{key}</option>
                        )
                    }
                </select>
                <button onClick={handleAddEffectEntryClick}>&#10133;</button>
            </div>
            <div>
                <hr />
            </div>
            <div className="workspace-effects-nodes">
                {
                    effectEntries.map((effectEntry, index) =>
                        <div key={effectEntry.id} className="workspace-effects-nodes-item">
                            <div className="workspace-effects-nodes-item-header">
                                {effectEntry.type}
                                <button onClick={() => handleRemoveEffectEntryClick(index)}>&#10134;</button>
                            </div>
                            <hr />
                            <div ref={el => {
                                if (el == null) {
                                    return;
                                }

                                const effectNodeControl = effectNodeControls.current[index];
                                effectNodeControl.control = el;
                                if (!effectNodeControl.isLoading) {
                                    initAudioNodeForEntry(effectEntry.type, effectNodeControl, el);
                                }
                            }}></div>
                        </div>
                    )
                }
            </div>
        </>
    );
}

export default EffectRack;