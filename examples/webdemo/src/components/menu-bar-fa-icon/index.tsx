import { IconProp } from "@fortawesome/fontawesome-svg-core";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import styles from "./styles.module.css";

function MenuBarFaIcon({ icon, label }: { icon: IconProp, label: string }) {
    return <div>
        <FontAwesomeIcon icon={icon} />
        <div className={styles["menu-bar-group-item-icon-text"]}>{label}</div>
    </div>;
}

export default MenuBarFaIcon;