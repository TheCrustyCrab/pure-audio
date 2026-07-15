import styles from "./styles.module.css";

type BeatProps = {
    isOn: boolean
};

function Beat(props: BeatProps) {
    return <div class={styles.beat}>
        <svg width={20} height={10} viewBox="0 0 20 10">            
            <use href={props.isOn ? "#beatOn" : "#beatOff"} />
        </svg>
    </div>;
}

export default Beat;