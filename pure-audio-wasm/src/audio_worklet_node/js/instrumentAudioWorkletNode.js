export let InstrumentAudioWorkletNode;
const root = (() => eval)()('this');
// AudioWorkletNode is available on the main thread but not on the audio thread
// Assign Dummy class to InstrumentAudioWorkletNode to avoid an import error from the audio thread
if (root.AudioWorkletNode === undefined) {
    InstrumentAudioWorkletNode = class Dummy { };
} else {
    InstrumentAudioWorkletNode = class InstrumentAudioWorkletNode extends AudioWorkletNode {
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
    };
}