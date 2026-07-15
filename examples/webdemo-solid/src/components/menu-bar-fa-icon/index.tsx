import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import styles from "./styles.module.css";
import FaIcon from "../fa-icon";

type MenuBarFaIconProps = {
    definition: IconDefinition,
    label: string
};

function MenuBarFaIcon(props: MenuBarFaIconProps) {
    return (
        <div>
            <FaIcon definition={props.definition} />
            <div class={styles["menu-bar-group-item-icon-text"]}>{props.label}</div>
        </div>
    );
}

export default MenuBarFaIcon;