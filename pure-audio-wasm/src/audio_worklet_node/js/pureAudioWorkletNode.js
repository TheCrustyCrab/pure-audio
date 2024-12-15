export let PureAudioWorkletNode;
const root = (() => eval)()('this');
// AudioWorkletNode is available on the main thread but not on the audio thread
// Assign Dummy class to PureAudioWorkletNode to avoid an import error from the audio thread
if (root.AudioWorkletNode === undefined) {
    PureAudioWorkletNode = class Dummy { };
} else {
    PureAudioWorkletNode = class PureAudioWorkletNode extends AudioWorkletNode {
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