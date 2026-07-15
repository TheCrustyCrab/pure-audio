import { SvgDefinitions } from "../svg-definitions";
import Beat from "./beat";
import styles from "./styles.module.css";

type BeatBarIndicatorProps = {
    currentBeat: number,
    beatsPerBar: number
};

function BeatBarIndicator(props: BeatBarIndicatorProps) {
    return <>
        <SvgDefinitions>
            <polygon id="beat" points="0,0 0,10 20,10 20,0" />

            <g id="beatOff" class={styles.rectangle}>
                <use href="#beat" />
            </g>

            <g id="beatOn" class={`${styles.rectangle} ${styles.on}`}>
                <use href="#beat" />
            </g>
        </SvgDefinitions>
        <div class={styles["beat-bar-indicator"]}>
            {
                [...Array(props.beatsPerBar)]
                    .map((_, i) => <Beat isOn={props.currentBeat === i} />)
            }
        </div>
    </>;
}

export default BeatBarIndicator;