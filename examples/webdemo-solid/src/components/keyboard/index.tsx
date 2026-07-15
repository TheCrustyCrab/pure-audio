import { createSignal, onCleanup, onMount } from "solid-js";
import { Octave } from "./octave";
import styles from "./styles.module.css";
import { createStore, produce } from "solid-js/store";
import { SvgDefinitions } from "../svg-definitions";
import type { EventBus } from "../../event-bus";

// https://stackoverflow.com/a/79762129
const PointerChordMode = {
    Note: "Note",
    Major: "Major",
    Minor: "Minor",
    Sus2: "Sus2",
    Sus4: "Sus4"
} as const;

type PointerChordMode = (typeof PointerChordMode)[keyof typeof PointerChordMode];

const chordKeyOffsets: { [key in PointerChordMode]: Array<number> } = {
    [PointerChordMode.Note]: [0],
    [PointerChordMode.Major]: [0, 4, 7],
    [PointerChordMode.Minor]: [0, 3, 7],
    [PointerChordMode.Sus2]: [0, 2, 7],
    [PointerChordMode.Sus4]: [0, 5, 7],
};

type KeyboardProps = {
    minOctave: number,
    octaveCount: number,
    scale: number,
    eventBus: EventBus
};

const SVG_KEY_WIDTH = 175;

export function Keyboard(props: KeyboardProps) {
    const [pointerChordMode, setPointerChordMode] = createSignal<PointerChordMode>(PointerChordMode.Note);
    const [showKeyLabels, setShowKeyLabels] = createSignal(false);
    const [canScrollLeft, setCanScrollLeft] = createSignal(false);
    const [canScrollRight, setCanScrollRight] = createSignal(false);
    const [activeNotes, setActiveNotes] = createStore<number[]>([]);

    // untracked
    let pointerActiveKey: number | null = null;
    let octavesDivRef!: HTMLDivElement;

    onMount(() => {
        props.eventBus.subscribe("midiNoteOff", handleMidiNoteOff);
        props.eventBus.subscribe("midiNoteOn", handleMidiNoteOn);
        document.body.addEventListener("pointerup", handlePointerUpOrCancel);
        document.body.addEventListener("pointercancel", handlePointerUpOrCancel);
        window.addEventListener("resize", handleWindowResize);
        handleWindowResize();
    });

    onCleanup(() => {
        props.eventBus.unsubscribe("midiNoteOff", handleMidiNoteOff);
        props.eventBus.unsubscribe("midiNoteOn", handleMidiNoteOn);
        document.body.removeEventListener("pointerup", handlePointerUpOrCancel);
        document.body.removeEventListener("pointercancel", handlePointerUpOrCancel);
        window.removeEventListener("resize", handleWindowResize);
    });

    const handleNoteOn = (key: number) => {
        pointerActiveKey = key;
        activateNotes(key);
    };

    const handleNoteEnter = (key: number) => {
        if (pointerActiveKey == null)
            return;

        pointerActiveKey = key;
        activateNotes(key);
    }

    const handleNoteLeave = () => {
        if (pointerActiveKey == null)
            return;

        stopAllNotes();
    }

    const handlePointerUpOrCancel = (_evt: PointerEvent) => {
        pointerActiveKey = null;
        stopAllNotes();
    };

    const activateNotes = (key: number) => {
        const keyOffsets = chordKeyOffsets[pointerChordMode()];
        const keys = keyOffsets.map(offset => key + offset);

        setActiveNotes(produce(current => {
            current.push(...keys);
        }));

        keys.forEach(key => props.eventBus.publish("noteOn", { key, velocity: 127 }));
    };

    // todo: keep separate array of midi notes to avoid interference with midi playback
    const stopAllNotes = () => {
        setActiveNotes(produce(currentNotes => {
            currentNotes.forEach(note => props.eventBus.publish("noteOff", { key: note, velocity: 127 }));
            currentNotes.splice(0);
        }));
    };

    const handleMidiNoteOff = ({ key }: { key: number }) => {
        setActiveNotes(current => current.filter(activeNote => activeNote !== key));
    };

    const handleMidiNoteOn = ({ key }: { key: number }) => {
        setActiveNotes(current => [...current, key]);
    };

    const handleScrollLeftClick = () => {
        const overflowWidth = octavesDivRef.scrollWidth - octavesDivRef.offsetWidth;
        const canScrollLeftAfterThisScroll = octavesDivRef.scrollLeft + SVG_KEY_WIDTH < overflowWidth;
        const canScrollRightAfterThisScroll = overflowWidth > 0 && octavesDivRef.scrollLeft + SVG_KEY_WIDTH > 0;
        // ideally, we should be able to lock the scroll buttons while the scrollBy animation lasts
        // unfortunately, there's no way to get notified when the animation ends, it doesn't trigger the onAnimationEnd event
        // so it's possible to spam the scroll buttons and get inconsistent behavior
        octavesDivRef.scrollBy({
            behavior: 'smooth',
            left: SVG_KEY_WIDTH
        });
        setCanScrollLeft(canScrollLeftAfterThisScroll);
        setCanScrollRight(canScrollRightAfterThisScroll);
    };

    const handleScrollRightClick = () => {
        const overflowWidth = octavesDivRef.scrollWidth - octavesDivRef.offsetWidth;
        const canScrollLeftAfterThisScroll = octavesDivRef.scrollLeft - SVG_KEY_WIDTH < overflowWidth;
        const canScrollRightAfterThisScroll = overflowWidth > 0 && octavesDivRef.scrollLeft - SVG_KEY_WIDTH > 0;
        octavesDivRef.scrollBy({
            behavior: 'smooth',
            left: -SVG_KEY_WIDTH
        });
        setCanScrollLeft(canScrollLeftAfterThisScroll);
        setCanScrollRight(canScrollRightAfterThisScroll);
    };

    const handleWindowResize = () => {
        const overflowWidth = octavesDivRef.scrollWidth - octavesDivRef.offsetWidth;
        const canScrollLeft = octavesDivRef.scrollLeft < overflowWidth;
        const canScrollRight = overflowWidth > 0 && octavesDivRef.scrollLeft > 0;
        setCanScrollLeft(canScrollLeft);
        setCanScrollRight(canScrollRight);
    };

    return (
        <div>
            <SvgDefinitions>
                <polygon id="whiteLeftKey" points="0,0 0,100 25,100 25,60 20,60 20,0" />
                <polygon id="whiteMiddleKey" points="5,0 5,60 0,60 0,100 25,100 25,60 20,60 20,0" />
                <polygon id="whiteRightKey" points="5,0 5,60 0,60 0,100 25,100 25,60 25,60 25,0" />
                <polygon id="blackKey" points="0,0 0,60 10,60 10,0" />

                <g id="whiteLeftKeyOff" class={styles.white}>
                    <use href="#whiteLeftKey" />
                </g>
                <g id="whiteLeftKeyOn" class={`${styles.white} ${styles.on}`}>
                    <use href="#whiteLeftKey" />
                </g>
                <g id="whiteMiddleKeyOff" class={styles.white}>
                    <use href="#whiteMiddleKey" />
                </g>
                <g id="whiteMiddleKeyOn" class={`${styles.white} ${styles.on}`}>
                    <use href="#whiteMiddleKey" />
                </g>
                <g id="whiteRightKeyOff" class={styles.white}>
                    <use href="#whiteRightKey" />
                </g>
                <g id="whiteRightKeyOn" class={`${styles.white} ${styles.on}`}>
                    <use href="#whiteRightKey" />
                </g>
                <g id="blackKeyOff" class={`${styles.black}`}>
                    <use href="#blackKey" />
                </g>
                <g id="blackKeyOn" class={`${styles.black} ${styles.on}`}>
                    <use href="#blackKey" />
                </g>
            </SvgDefinitions>
            <div>
                Chord mode:
                {
                    [PointerChordMode.Note, PointerChordMode.Major, PointerChordMode.Minor, PointerChordMode.Sus2, PointerChordMode.Sus4].map(mode =>
                        <>
                            <input type="radio" id={mode} checked={pointerChordMode() === mode} onChange={() => setPointerChordMode(mode)} />
                            <label for={mode}>{mode}</label>
                        </>
                    )
                }
            </div>
            <div>
                <input type="checkbox" id="toggleKeyLabels" onChange={evt => setShowKeyLabels(evt.target.checked)} />
                <label for="toggleKeyLabels">Key labels</label>
            </div>
            <div class={styles.keyboard}>
                <div class={styles["keyboard-octaves-scroll-arrow"]} style={{ visibility: canScrollRight() ? "visible" : "hidden" }} onClick={handleScrollRightClick}>
                    &lt;
                </div>
                <div ref={octavesDivRef} class={styles["keyboard-octaves"]}>
                    {
                        // the octaveCount is fixed so no reactive rendering (For/Index) is needed
                        [...Array(props.octaveCount)].map((_, i) =>
                            <Octave index={i + props.minOctave} onNoteOn={handleNoteOn} activeNotes={activeNotes}
                                showKeyLabels={showKeyLabels()} scale={props.scale} onNoteEnter={handleNoteEnter} onNoteLeave={handleNoteLeave} />
                        )
                    }
                </div>
                <div class={styles["keyboard-octaves-scroll-arrow"]} style={{ visibility: canScrollLeft() ? "visible" : "hidden" }} onClick={handleScrollLeftClick}>
                    &gt;
                </div>
            </div>
        </div>
    );
}