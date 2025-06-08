import { useEffect, useRef } from "react";

// https://overreacted.io/making-setinterval-declarative-with-react-hooks/
function useInterval(callback: () => void, delay: number | null) {
    const savedCallback = useRef<() => void>(null!);

    useEffect(() => {
        savedCallback.current = callback;
    }, [callback]);

    useEffect(() => {
        function tick() {
            savedCallback.current();
        }

        if (delay !== null) {
            const id = setInterval(tick, delay);
            return () => clearInterval(id);
        }
    }, [delay]);
}

export default useInterval;