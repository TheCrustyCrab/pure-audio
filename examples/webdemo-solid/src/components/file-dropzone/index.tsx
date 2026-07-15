import { createSignal } from "solid-js";
import styles from "./styles.module.css";
import type { JSX } from "solid-js/jsx-runtime";

type DraggingFileState = "None" | "InvalidFile" | "ValidFile";

type FileDropzoneProps = {
    acceptedTypes: string[],
    onFileDrop: (file: File) => void,
    children: JSX.Element
};

function FileDropzone(props: FileDropzoneProps) {
    const [draggingFileState, setDraggingFileState] = createSignal<DraggingFileState>("None");
    const draggingFileStateCssClass = () => {
        switch (draggingFileState()) {
            case "None":
                return "";
            case "InvalidFile":
                return styles["dragging-file-invalid"];
            case "ValidFile":
                return styles["dragging-file-valid"];
        }
    };
    let dragEnterCount = 0;

    const getSingleDataTransfer = (evt: DragEvent): { item: DataTransferItem | null, file: File | null } | null => {
        if (!evt.dataTransfer) {
            return null;
        }

        if (evt.dataTransfer.items) {
            if (evt.dataTransfer.items.length != 1) {
                return null;
            }

            const item = evt.dataTransfer.items[0];
            if (item.kind !== "file" || !props.acceptedTypes.find(type => type === item.type)) {
                return null;
            }

            return { item, file: item.getAsFile() };
        } else {
            if (evt.dataTransfer.files.length != 1) {
                return null;
            }

            const file = evt.dataTransfer.files[0];
            if (!props.acceptedTypes.find(type => type === file.type)) {
                return null;
            }

            return { item: null, file };
        }
    }

    const handleDragEnter: JSX.EventHandler<HTMLDivElement, DragEvent> = evt => {
        dragEnterCount++;
        setDraggingFileState(getSingleDataTransfer(evt) === null ? "InvalidFile" : "ValidFile");
    };

    const handleDragLeave: JSX.EventHandler<HTMLDivElement, DragEvent> = () => {
        if (--dragEnterCount === 0) {
            setDraggingFileState("None");
        }
    };

    const handleDrop: JSX.EventHandler<HTMLDivElement, DragEvent> = async evt => {
        evt.preventDefault();
        setDraggingFileState("None");

        const itemOrFile = getSingleDataTransfer(evt);
        if (itemOrFile === null) {
            return;
        }

        let { item, file } = itemOrFile;

        props.onFileDrop(file ?? item!.getAsFile()!);
    };

    return (
        <div class={draggingFileStateCssClass()} onDragOver={evt => evt.preventDefault()} onDragEnter={handleDragEnter} onDragLeave={handleDragLeave} onDrop={handleDrop}>
            {props.children}
        </div>
    );
}

export default FileDropzone;