import { PropsWithChildren, useMemo, useRef, useState } from "react";
import styles from "./styles.module.css";

enum DraggingFileState {
    None,
    InvalidFile,
    ValidFile
}

interface FileDropzoneProps {
    acceptedTypes: string[],
    onFileDrop: (file: File) => void
}

function FileDropzone({ acceptedTypes, onFileDrop, children }: PropsWithChildren<FileDropzoneProps>) {
    const [draggingFileState, setDraggingFileState] = useState(DraggingFileState.None);
    const draggingFileStateCssClass = useMemo(() => {
        switch (draggingFileState) {
            case DraggingFileState.None:
                return "";
            case DraggingFileState.InvalidFile:
                return styles["dragging-file-invalid"];
            case DraggingFileState.ValidFile:
                return styles["dragging-file-valid"];
        }
    }, [draggingFileState]);
    const dragEnterCount = useRef(0);

    const getSingleDataTransfer = (evt: React.DragEvent): { item: DataTransferItem | null, file: File | null } | null => {
        if (!evt.dataTransfer) {
            return null;
        }

        if (evt.dataTransfer.items) {
            if (evt.dataTransfer.items.length != 1) {
                return null;
            }

            const item = evt.dataTransfer.items[0];
            if (item.kind !== "file" || !acceptedTypes.find(type => type === item.type)) {
                return null;
            }

            return { item, file: item.getAsFile() };
        } else {
            if (evt.dataTransfer.files.length != 1) {
                return null;
            }

            const file = evt.dataTransfer.files[0];
            if (!acceptedTypes.find(type => type === file.type)) {
                return null;
            }

            return { item: null, file };
        }
    }

    const handleDragEnter = (evt: React.DragEvent) => {
        dragEnterCount.current++;
        setDraggingFileState(getSingleDataTransfer(evt) === null ? DraggingFileState.InvalidFile : DraggingFileState.ValidFile);
    };

    const handleDragLeave = (_evt: React.DragEvent) => {
        if (--dragEnterCount.current === 0) {
            setDraggingFileState(DraggingFileState.None);
        }
    };

    const handleDrop = async (evt: React.DragEvent) => {
        evt.preventDefault();
        setDraggingFileState(DraggingFileState.None);

        const itemOrFile = getSingleDataTransfer(evt);
        if (itemOrFile === null) {
            return;
        }

        let { item, file } = itemOrFile;
        
        onFileDrop(file ?? item!.getAsFile()!);
    };

    return (
        <div className={draggingFileStateCssClass} onDragOver={evt => evt.preventDefault()} onDragEnter={handleDragEnter} onDragLeave={handleDragLeave} onDrop={handleDrop}>
            {children}
        </div>
    );
}

export default FileDropzone;