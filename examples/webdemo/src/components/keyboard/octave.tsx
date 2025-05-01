import styles from "./styles.module.css";

export function Octave({ index, onNoteOn, activeNotes }: { index: number, onNoteOn: (key: number) => void, activeNotes: number[] }) {    
    const renderKeySvg = (keySvgBaseId: string, x: number, keyOffset: number) => {
        const key = index * 12 + keyOffset;
        const svgLinkHref = `#${keySvgBaseId}${activeNotes.findIndex(activeNote => activeNote === key) !== -1 ? "On" : "Off" }`
        return <use key={key} data-key={key} xlinkHref={svgLinkHref} x={x} y={0} onPointerDown={() => onNoteOn(key)} />
    }

    const keyPositions = [
        { keySvgBaseId: "whiteLeftKey", x: 0 },
        { keySvgBaseId: "blackKey", x: 20, },
        { keySvgBaseId: "whiteMiddleKey", x: 25 },
        { keySvgBaseId: "blackKey", x: 45 },
        { keySvgBaseId: "whiteRightKey", x: 50},
        { keySvgBaseId: "whiteLeftKey", x: 75 },
        { keySvgBaseId: "blackKey", x: 95 },
        { keySvgBaseId: "whiteMiddleKey", x: 100 },
        { keySvgBaseId: "blackKey", x: 120 },
        { keySvgBaseId: "whiteMiddleKey", x: 125 },
        { keySvgBaseId: "blackKey", x: 145 },
        { keySvgBaseId: "whiteRightKey", x: 150 }
    ];

    return <svg width={175} height={100}>
        <defs>
            <polygon id="whiteLeftKey" points="0,0 0,100 25,100 25,60 20,60 20,0" />
            <polygon id="whiteMiddleKey" points="5,0 5,60 0,60 0,100 25,100 25,60 20,60 20,0" />
            <polygon id="whiteRightKey" points="5,0 5,60 0,60 0,100 25,100 25,60 25,60 25,0" />
            <polygon id="blackKey" points="0,0 0,60 10,60 10,0" />

            <g id="whiteLeftKeyOff" className={styles.white}>
                <use xlinkHref="#whiteLeftKey"/>
            </g>
            <g id="whiteLeftKeyOn" className={`${styles.white} ${styles.on}`}>
                <use xlinkHref="#whiteLeftKey"/>
            </g>
            <g id="whiteMiddleKeyOff" className={styles.white}>
                <use xlinkHref="#whiteMiddleKey"/>
            </g>
            <g id="whiteMiddleKeyOn" className={`${styles.white} ${styles.on}`}>
                <use xlinkHref="#whiteMiddleKey"/>
            </g>
            <g id="whiteRightKeyOff" className={styles.white}>
                <use xlinkHref="#whiteRightKey"/>
            </g>
            <g id="whiteRightKeyOn" className={`${styles.white} ${styles.on}`}>
                <use xlinkHref="#whiteRightKey"/>
            </g>
            <g id="blackKeyOff" className={`${styles.black}`}>
                <use xlinkHref="#blackKey"/>
            </g>
            <g id="blackKeyOn" className={`${styles.black} ${styles.on}`}>
                <use xlinkHref="#blackKey"/>
            </g>
        </defs>
        {
            keyPositions.map((keyPosition, index) => renderKeySvg(keyPosition.keySvgBaseId, keyPosition.x, index))
        }
    </svg>;
}