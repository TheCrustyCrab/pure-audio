import { Show } from "solid-js";

type KeyProps = {
    svgBaseId: string,
    x: number,
    isOn: boolean,
    showKeyLabels: boolean,
    label: string,
    onPointerDown: () => void,
    onMouseEnter: () => void,
    onMouseLeave: () => void
};

export function Key(props: KeyProps) {
    return (
        <>
            <use href={`#${props.svgBaseId}${props.isOn ? "On" : "Off"}`}
                x={props.x} y={0} onPointerDown={props.onPointerDown}
                onMouseEnter={props.onMouseEnter} onMouseLeave={props.onMouseLeave} />
            <Show when={props.showKeyLabels && props.svgBaseId.startsWith("white")}>
                <text x={props.x + 3} y={90}>{props.label}</text>
            </Show>
        </>
    );
}