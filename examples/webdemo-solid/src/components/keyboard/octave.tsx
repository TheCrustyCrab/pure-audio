import styles from "./styles.module.css";
import { Key } from "./key";

const keyPositions = [
    { keySvgBaseId: "whiteLeftKey", x: 0 },
    { keySvgBaseId: "blackKey", x: 20, },
    { keySvgBaseId: "whiteMiddleKey", x: 25 },
    { keySvgBaseId: "blackKey", x: 45 },
    { keySvgBaseId: "whiteRightKey", x: 50 },
    { keySvgBaseId: "whiteLeftKey", x: 75 },
    { keySvgBaseId: "blackKey", x: 95 },
    { keySvgBaseId: "whiteMiddleKey", x: 100 },
    { keySvgBaseId: "blackKey", x: 120 },
    { keySvgBaseId: "whiteMiddleKey", x: 125 },
    { keySvgBaseId: "blackKey", x: 145 },
    { keySvgBaseId: "whiteRightKey", x: 150 }
];

const keyNames = [
    "C",
    "C#",
    "D",
    "D#",
    "E",
    "F",
    "F#",
    "G",
    "G#",
    "A",
    "A#",
    "B"
];

type OctaveProps = {
    index: number,
    onNoteOn: (key: number) => void,
    onNoteEnter: (key: number) => void,
    onNoteLeave: (key: number) => void,
    activeNotes: number[],
    scale: number,
    showKeyLabels: boolean
};

export function Octave(props: OctaveProps) {
    return (
        <div class={styles["keyboard-octave"]}>
            <svg width={175 * props.scale} height={100 * props.scale} viewBox="0 0 175 100">
                {
                    keyPositions.map((keyPosition, index) => {
                        const note = props.index * 12 + index;
                        return <Key svgBaseId={keyPosition.keySvgBaseId}
                            x={keyPosition.x}
                            isOn={props.activeNotes.findIndex(activeNote => activeNote === note) !== -1}
                            showKeyLabels={props.showKeyLabels}
                            label={`${keyNames[index]}${props.index}`}
                            onPointerDown={() => props.onNoteOn(note)}
                            onMouseEnter={() => props.onNoteEnter(note)}
                            onMouseLeave={() => props.onNoteLeave(note)} />
                    })
                }
            </svg>
        </div>
    );
}