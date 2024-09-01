export class InstrumentAudioWorkletNode extends AudioWorkletNode {
    noteOn() {
        this.port.postMessage({
            type: "noteOn",
            // todo: key, channel, ...
        });
    }

    noteOff() {
        this.port.postMessage({
            type: "noteOff",
            // todo: key, channel, ...
        });
    }
}