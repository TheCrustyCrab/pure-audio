import type { JSX } from "solid-js/jsx-runtime";

type SvgDefinitionProps = {
    children: JSX.Element
};

export function SvgDefinitions(props: SvgDefinitionProps) {
    return <svg style="display: none">
        <defs>
            {props.children}
        </defs>
    </svg>;
};