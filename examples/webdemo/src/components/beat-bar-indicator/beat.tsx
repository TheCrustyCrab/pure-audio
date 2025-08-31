import styles from "./styles.module.css";

function Beat({ isOn }: { isOn: boolean }) {
    return <div className={styles.beat}>
        <svg width={20} height={10} viewBox="0 0 20 10">            
            <use xlinkHref={isOn ? "#beatOn" : "#beatOff"} />
        </svg>
    </div>
}

export default Beat;