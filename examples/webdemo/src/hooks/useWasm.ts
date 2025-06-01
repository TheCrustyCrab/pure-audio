import { useEffect, useState } from "react";

export default function useWasm<TInput, TOutput>(init: (input?: TInput) => Promise<TOutput>, dispose?: () => void) {
    const [loaded, setLoaded] = useState(false);

    useEffect(() => {
        const load = async () => {
            await init();
            setLoaded(true);
        }

        load();

        return () => {
            setLoaded(false);
            if (dispose) {
                dispose();
            }
        }
    }, []);

    return loaded;
}