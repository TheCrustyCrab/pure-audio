import { createContext, useContext } from "react";

export class AudioGraph {
    private readonly _audioContext: AudioContext;
    private readonly _effectNodes: AudioWorkletNode[];
    private _synthNode: AudioNode | null;
    private _synthDestinationNode: AudioNode;

    constructor(audioContext: AudioContext) {
        this._audioContext = audioContext;
        this._effectNodes = [];
        this._synthNode = null;
        this._synthDestinationNode = audioContext.destination;
    }

    get audioContext(): AudioContext {
        return this._audioContext;
    }

    get currentTime(): number {
        return this._audioContext.currentTime;
    }

    setSynth(node: AudioWorkletNode) {
        this._synthNode = node;
        node.connect(this._synthDestinationNode);
    }

    appendEffect(node: AudioWorkletNode) {
        const effectCount = this._effectNodes.length;
        if (effectCount === 0) {
            this._synthNode?.disconnect(this._synthDestinationNode);
            this._synthDestinationNode = node;
            this._synthNode?.connect(this._synthDestinationNode);
        } else {
            const previousEffectNode = this._effectNodes[this._effectNodes.length - 1];
            previousEffectNode.disconnect(this._audioContext.destination);
            previousEffectNode.connect(node);
        }

        this._effectNodes.push(node);
        node.connect(this._audioContext.destination);
    }

    removeEffect(node: AudioWorkletNode) {
        const effectIndex = this._effectNodes.indexOf(node);
        if (effectIndex === 0) {
            this._synthNode?.disconnect(this._synthDestinationNode);
            const nextEffectNode = this._effectNodes[effectIndex + 1];
            if (nextEffectNode) {
                this._synthDestinationNode = nextEffectNode;
            } else {
                this._synthDestinationNode = this._audioContext.destination;
            }

            this._synthNode?.connect(this._synthDestinationNode);
        } else {
            const previousEffectNode = this._effectNodes[effectIndex - 1];
            previousEffectNode.disconnect(node);

            const nextEffectNode = this._effectNodes[effectIndex + 1];
            if (nextEffectNode) {
                previousEffectNode.connect(nextEffectNode);
            } else {
                previousEffectNode.connect(this._audioContext.destination);
            }
        }

        this._effectNodes.splice(effectIndex, 1);
    }
}

export const AudioGraphContext = createContext<AudioGraph | null>(null);

function useAudioGraph() {
    return useContext(AudioGraphContext);
}

export default useAudioGraph;