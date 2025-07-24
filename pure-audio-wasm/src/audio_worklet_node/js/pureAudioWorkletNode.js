export let PureAudioWorkletNode;
const root = (() => eval)()('this');
// AudioWorkletNode is available on the main thread but not on the audio thread
// Assign Dummy class to PureAudioWorkletNode to avoid an import error from the audio thread
if (root.AudioWorkletNode === undefined) {
    PureAudioWorkletNode = class Dummy { };
} else {
    let finalizationRegistry;
    PureAudioWorkletNode = class PureAudioWorkletNode extends AudioWorkletNode {
        constructor(context, name, options, wasm, parameterIndexNameMap) {
            super(context, name, options);
            this.outputEventListeners = [];
            this.wasm = wasm;
            this.parameterIndexNameMap = parameterIndexNameMap;
            finalizationRegistry = (typeof FinalizationRegistry === 'undefined')
                ? { register: () => {}, unregister: () => {} }
                : new FinalizationRegistry(ptr => wasm.destroyRawWasmParameterConverter(ptr));
            this.rawParameterConverterPtr = wasm.createRawWasmParameterConverter();
            finalizationRegistry.register(this, this.rawParameterConverterPtr);

            this.port.onmessage = msg => {
                if (msg.data.type === "outputEvent") {
                    this.outputEventListeners.forEach(callback => {
                        callback(msg.data.data);
                    });
                }
            }
        }
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

        scheduleNoteOn(time, key, velocity) {
            this.port.postMessage({
                type: "scheduleNoteOn",
                data: {
                    time,
                    key,
                    velocity
                }
            });
        }

        scheduleNoteOff(time, key, velocity) {
            this.port.postMessage({
                type: "scheduleNoteOff",
                data: {
                    time,
                    key,
                    velocity
                }
            });
        }

        setHostTempo(value) {
            this.port.postMessage({
                type: "setHostTempo",
                data: {
                    value
                }
            });
        }

        setHostIsPlaying(value) {
            this.port.postMessage({
                type: "setHostIsPlaying",
                data: {
                    value
                }
            });
        }

        addOutputEventListener(callback) {
            this.outputEventListeners.push(callback);
        }

        requestStop() {
            this.port.postMessage({
                type: "requestStop"
            });
        }

        getParameterText(key) {
            const parameter = this.parameters.get(key);
            if (parameter === undefined) {
                throw new Error("parameter not found");
            }
            
            const index = this.parameterIndexNameMap.get(key);
            return this.wasm.valueToText(this.rawParameterConverterPtr, index, parameter.value);
        }

        __getRawParameterConverterPtr() {
            return this.rawParameterConverterPtr;
        }
    };
}