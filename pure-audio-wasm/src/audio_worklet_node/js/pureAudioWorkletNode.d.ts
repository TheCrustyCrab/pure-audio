export class PureAudioWorkletNode extends AudioWorkletNode {
    noteOn(key: number, velocity: number): void;
    noteOff(key: number, velocity: number): void;
    addOutputEventListener(callback: (event: any) => void): void;
    requestStop(): void;
}