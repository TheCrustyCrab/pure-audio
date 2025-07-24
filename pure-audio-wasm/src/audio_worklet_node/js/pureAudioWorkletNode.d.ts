export class PureAudioWorkletNode extends AudioWorkletNode {
    noteOn(key: number, velocity: number): void;
    noteOff(key: number, velocity: number): void;
    scheduleNoteOn(time: number, key: number, velocity: number): void;
    scheduleNoteOff(time: number, key: number, velocity: number): void;
    setHostTempo(value: number): void;
    setHostIsPlaying(value: boolean): void;
    addOutputEventListener(callback: (event: any) => void): void;
    requestStop(): void;
    getParameterText(key: string): string;
}