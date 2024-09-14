export class InstrumentAudioWorkletNode extends AudioWorkletNode {
    noteOn(key) {
        this.port.postMessage({
            type: "noteOn",
            data: {
                key
            }
            // todo: channel, ...
        });
    }

    noteOff(key) {
        this.port.postMessage({
            type: "noteOff",
            data: {
                key
            }
            // todo: channel, ...
        });
    }
}