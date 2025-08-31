import SvgDefinitions from "../svg-definitions";
import Beat from "./beat";
import styles from "./styles.module.css";

function BeatBarIndicator({ currentBeat, beatsPerBar }: { currentBeat: number, beatsPerBar: number }) {
    return <>
        <SvgDefinitions>
            <polygon id="beat" points="0,0 0,10 20,10 20,0" />

            <g id="beatOff" className={styles.rectangle}>
                <use xlinkHref="#beat" />
            </g>

            <g id="beatOn" className={`${styles.rectangle} ${styles.on}`}>
                <use xlinkHref="#beat" />
            </g>
        </SvgDefinitions>
        <div className={styles["beat-bar-indicator"]}>
            {
                [...Array(beatsPerBar)]
                    .map((_, i) => <Beat key={i} isOn={currentBeat === i} />)
            }
        </div>
    </>;
}

export default BeatBarIndicator;