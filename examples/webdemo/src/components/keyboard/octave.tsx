import { Fragment } from "react/jsx-runtime";
import styles from "./styles.module.css";

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

export function Octave({ index, onNoteOn, activeNotes, scale, showKeyLabels }: { index: number, onNoteOn: (key: number) => void, activeNotes: number[], scale: number, showKeyLabels: boolean }) {
    const renderKeySvg = (keySvgBaseId: string, x: number, keyOffset: number) => {
        const key = index * 12 + keyOffset;
        const svgLinkHref = `#${keySvgBaseId}${activeNotes.findIndex(activeNote => activeNote === key) !== -1 ? "On" : "Off"}`
        return <Fragment key={key}>
            <use data-key={key} xlinkHref={svgLinkHref} x={x} y={0} onPointerDown={() => onNoteOn(key)} />
            {showKeyLabels && keySvgBaseId.startsWith("white") ? <text x={x + 3} y={90}>{`${keyNames[keyOffset]}${index}`}</text> : null}
        </Fragment>;
    }

    return <div className={styles["keyboard-octave"]}>
        <svg width={175 * scale} height={100 * scale} viewBox="0 0 175 100">
            {
                keyPositions.map((keyPosition, index) => renderKeySvg(keyPosition.keySvgBaseId, keyPosition.x, index))
            }
        </svg>
    </div>;
}