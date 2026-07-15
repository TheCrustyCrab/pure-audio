import { icon, library, type IconDefinition } from "@fortawesome/fontawesome-svg-core"
import { template } from "solid-js/web";

type FaIconProps = {
    definition: IconDefinition
};

export default function FaIcon(props: FaIconProps) {
    library.add(props.definition);
    const elementFn = template(icon({ prefix: props.definition.prefix, iconName: props.definition.iconName }).html[0], false, false);
    return elementFn();
};