// the timing from setInterval on a separate worker (thread) is more reliable than on the main thread
// see https://web.dev/articles/audio-scheduling
let timerId = null;

type ScheduleMessage = { type: "start", interval: number } | { type: "stop" };

self.onmessage = (evt: MessageEvent<ScheduleMessage>) => {
    if (evt.data.type === "start") {
        timerId = setInterval(() => self.postMessage("tick"), evt.data.interval);
    } else if (evt.data.type === "stop") {
        clearInterval(timerId!);
        timerId = null;
    }
};

export {};