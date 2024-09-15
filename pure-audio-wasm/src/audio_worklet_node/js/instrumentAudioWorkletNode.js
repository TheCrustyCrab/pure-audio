export class InstrumentAudioWorkletNode extends AudioWorkletNode {
    noteOn(key, velocity) {
        this.port.postMessage({
            type: "noteOn",
            data: {
                key,
                velocity
            }
        });
    }

    noteOff(key, velocity) {
        this.port.postMessage({
            type: "noteOff",
            data: {
                key,
                velocity
            }
        });
    }
}