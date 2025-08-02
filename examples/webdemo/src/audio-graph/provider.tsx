import { ReactNode, useRef, useState } from "react";
import { AudioGraph, AudioGraphContext } from ".";
import styles from "./styles.module.css"

enum InitializationState {
    Uninitialized,
    Initializing,
    Initialized
}

export const AudioGraphProvider = ({ children }: { children: ReactNode }) => {
    const audioGraph = useRef<AudioGraph>(null);
    const [initializationState, setInitializationState] = useState<InitializationState>(InitializationState.Uninitialized);

    const initAudio = async () => {
        audioGraph.current = new AudioGraph(new AudioContext());
        setInitializationState(InitializationState.Initializing);
    };

    return (
        <div className={styles.container}>
            {
                initializationState !== InitializationState.Initialized
                    ? <div className={`${styles["preinit-overlay"]} ${initializationState === InitializationState.Initializing ? styles.hiding : ""}`}
                        onClick={initAudio} onAnimationEnd={() => setInitializationState(InitializationState.Initialized)}>
                        <p>Click to start audio</p>
                    </div>
                    : null
            }
            <AudioGraphContext.Provider value={audioGraph.current}>
                {children}
            </AudioGraphContext.Provider>
        </div>
    )
}