import { Fragment, useCallback, useEffect, useRef, useState } from 'react';
import { Octave } from './octave';
import useEventBus from '../../hooks/useEventBus';
import styles from './styles.module.css';
import SvgDefinitions from '../svg-definitions';

interface KeyboardProps {
    minOctave: number,
    octaveCount: number,
    scale: number
}

enum PointerChordMode {
    Note = "Note",
    Major = "Major",
    Minor = "Minor",
    Sus2 = "Sus2",
    Sus4 = "Sus4"
}

const chordKeyOffsets: { [key in PointerChordMode]: Array<number> } = {
    [PointerChordMode.Note]: [0],
    [PointerChordMode.Major]: [0, 4, 7],
    [PointerChordMode.Minor]: [0, 3, 7],
    [PointerChordMode.Sus2]: [0, 2, 7],
    [PointerChordMode.Sus4]: [0, 5, 7]
}

const SVG_KEY_WIDTH = 175;

function Keyboard({ minOctave, octaveCount, scale }: KeyboardProps) {
    const [pointerChordMode, setPointerChordMode] = useState<PointerChordMode>(PointerChordMode.Note);
    const pointerActiveKey = useRef<number>(null);
    const [activeNotes, setActiveNotes] = useState<number[]>([]);
    const [showKeyLabels, setShowKeyLabels] = useState(false);
    const [canScrollLeft, setCanScrollLeft] = useState(true);
    const [canScrollRight, setCanScrollRight] = useState(true);
    const octavesElement = useRef<HTMLDivElement>(null);
    const eventBus = useEventBus();

    useEffect(() => {
        eventBus.subscribe("midiNoteOff", handleMidiNoteOff);
        eventBus.subscribe("midiNoteOn", handleMidiNoteOn);
        window.addEventListener("resize", handleWindowResize);
        handleWindowResize();

        return () => {
            eventBus.unsubscribe("midiNoteOff", handleMidiNoteOff);
            eventBus.unsubscribe("midiNoteOn", handleMidiNoteOn);
            window.removeEventListener("resize", handleWindowResize);
        };
    }, []);

    useEffect(() => {
        document.body.addEventListener("pointerup", handlePointerUpOrCancel);
        document.body.addEventListener("pointercancel", handlePointerUpOrCancel);

        return () => {
            document.body.removeEventListener("pointerup", handlePointerUpOrCancel);
            document.body.removeEventListener("pointercancel", handlePointerUpOrCancel);
        };
    }, [minOctave, octaveCount, activeNotes]);

    const handleNoteOn = (key: number) => {
        document.body.addEventListener("pointermove", handlePointerMove);
        pointerActiveKey.current = key;
        const keyOffsets = chordKeyOffsets[pointerChordMode];
        const keys = keyOffsets.map(offset => key + offset);
        setActiveNotes([...activeNotes, ...keys]);
        keys.forEach(key => eventBus.publish("noteOn", { key, velocity: 127 }));
    };

    const handlePointerUpOrCancel = (_evt: PointerEvent) => {
        document.body.removeEventListener("pointermove", handlePointerMove);
        setActiveNotes(currentActiveNotes => {
            currentActiveNotes.forEach(activeNote => {
                eventBus.publish("noteOff", { key: activeNote, velocity: 127 })
            });
            return [];
        });
    };

    const handlePointerMove = useCallback((evt: PointerEvent) => {
        const newKey = (evt.target as SVGUseElement).getAttribute("data-key");
        if (newKey === null) {
            return;
        }

        const newKeyNumber = parseInt(newKey);
        if (newKeyNumber === pointerActiveKey.current) {
            return;
        }

        const keyOffsets = chordKeyOffsets[pointerChordMode];
        const oldKeys = keyOffsets.map(offset => pointerActiveKey.current! + offset);
        oldKeys.forEach(oldKey => {
            eventBus.publish("noteOff", { key: oldKey, velocity: 127 })
        });

        const newKeys = keyOffsets.map(offset => newKeyNumber + offset);
        newKeys.forEach(newKey => {
            eventBus.publish("noteOn", { key: newKey, velocity: 127 })
        });
        setActiveNotes(newKeys);
        pointerActiveKey.current = newKeyNumber;
    }, [pointerChordMode]);

    const handleMidiNoteOff = ({ key }: { key: number }) => {
        setActiveNotes(current => current.filter(activeNote => activeNote !== key));
    }

    const handleMidiNoteOn = ({ key }: { key: number }) => {
        setActiveNotes(current => [...current, key]);
    }

    const handleScrollLeftClick = () => {
        const overflowWidth = octavesElement.current!.scrollWidth - octavesElement.current!.offsetWidth;
        const canScrollLeftAfterThisScroll = octavesElement.current!.scrollLeft + SVG_KEY_WIDTH < overflowWidth;
        const canScrollRightAfterThisScroll = overflowWidth > 0 && octavesElement.current!.scrollLeft + SVG_KEY_WIDTH > 0;
        // ideally, we should be able to lock the scroll buttons while the scrollBy animation lasts
        // unfortunately, there's no way to get notified when the animation ends, it doesn't trigger the onAnimationEnd event
        // so it's possible to spam the scroll buttons and get inconsistent behavior
        octavesElement.current!.scrollBy({
            behavior: 'smooth',
            left: SVG_KEY_WIDTH
        });
        setCanScrollLeft(canScrollLeftAfterThisScroll);
        setCanScrollRight(canScrollRightAfterThisScroll);
    }

    const handleScrollRightClick = () => {
        const overflowWidth = octavesElement.current!.scrollWidth - octavesElement.current!.offsetWidth;
        const canScrollLeftAfterThisScroll = octavesElement.current!.scrollLeft - SVG_KEY_WIDTH < overflowWidth;
        const canScrollRightAfterThisScroll = overflowWidth > 0 && octavesElement.current!.scrollLeft - SVG_KEY_WIDTH > 0;
        octavesElement.current!.scrollBy({
            behavior: 'smooth',
            left: -SVG_KEY_WIDTH
        });
        setCanScrollLeft(canScrollLeftAfterThisScroll);
        setCanScrollRight(canScrollRightAfterThisScroll);
    }

    const handleWindowResize = () => {
        const overflowWidth = octavesElement.current!.scrollWidth - octavesElement.current!.offsetWidth;
        const canScrollLeft = octavesElement.current!.scrollLeft < overflowWidth;
        const canScrollRight = overflowWidth > 0 && octavesElement.current!.scrollLeft > 0;
        setCanScrollLeft(canScrollLeft);
        setCanScrollRight(canScrollRight);
    }

    return (
        <div>
            <SvgDefinitions>
                <polygon id="whiteLeftKey" points="0,0 0,100 25,100 25,60 20,60 20,0" />
                <polygon id="whiteMiddleKey" points="5,0 5,60 0,60 0,100 25,100 25,60 20,60 20,0" />
                <polygon id="whiteRightKey" points="5,0 5,60 0,60 0,100 25,100 25,60 25,60 25,0" />
                <polygon id="blackKey" points="0,0 0,60 10,60 10,0" />

                <g id="whiteLeftKeyOff" className={styles.white}>
                    <use xlinkHref="#whiteLeftKey" />
                </g>
                <g id="whiteLeftKeyOn" className={`${styles.white} ${styles.on}`}>
                    <use xlinkHref="#whiteLeftKey" />
                </g>
                <g id="whiteMiddleKeyOff" className={styles.white}>
                    <use xlinkHref="#whiteMiddleKey" />
                </g>
                <g id="whiteMiddleKeyOn" className={`${styles.white} ${styles.on}`}>
                    <use xlinkHref="#whiteMiddleKey" />
                </g>
                <g id="whiteRightKeyOff" className={styles.white}>
                    <use xlinkHref="#whiteRightKey" />
                </g>
                <g id="whiteRightKeyOn" className={`${styles.white} ${styles.on}`}>
                    <use xlinkHref="#whiteRightKey" />
                </g>
                <g id="blackKeyOff" className={`${styles.black}`}>
                    <use xlinkHref="#blackKey" />
                </g>
                <g id="blackKeyOn" className={`${styles.black} ${styles.on}`}>
                    <use xlinkHref="#blackKey" />
                </g>
            </SvgDefinitions>
            <div>
                Chord mode:
                {
                    [PointerChordMode.Note, PointerChordMode.Major, PointerChordMode.Minor, PointerChordMode.Sus2, PointerChordMode.Sus4].map(mode =>
                        <Fragment key={mode}>
                            <input type="radio" id={mode} checked={pointerChordMode === mode} onChange={() => setPointerChordMode(mode)} />
                            <label htmlFor={mode}>{mode}</label>
                        </Fragment>
                    )
                }
            </div>
            <div>
                <input type="checkbox" id="toggleKeyLabels" onChange={evt => setShowKeyLabels(evt.target.checked)} />
                <label htmlFor="toggleKeyLabels">Key labels</label>
            </div>
            <hr />
            <div className={styles.keyboard}>
                <div className={styles["keyboard-octaves-scroll-arrow"]} style={{ visibility: canScrollRight ? "visible" : "hidden" }} onClick={handleScrollRightClick}>
                    &lt;
                </div>
                <div ref={octavesElement} className={styles["keyboard-octaves"]}>
                    {
                        [...Array(octaveCount)].map((_, i) => {
                            const octaveIndex = i + minOctave;
                            return <Octave key={octaveIndex} index={octaveIndex} onNoteOn={handleNoteOn} activeNotes={activeNotes} scale={scale} showKeyLabels={showKeyLabels} />;
                        })
                    }
                </div>
                <div className={styles["keyboard-octaves-scroll-arrow"]} style={{ visibility: canScrollLeft ? "visible" : "hidden" }} onClick={handleScrollLeftClick}>
                    &gt;
                </div>
            </div>
        </div>
    )
}

export default Keyboard;