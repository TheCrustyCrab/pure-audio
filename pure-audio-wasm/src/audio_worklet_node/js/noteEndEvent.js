export class NoteEndEvent {
    constructor(portIndex, channel, key, noteId) {
        this.portIndex = portIndex;
        this.channel = channel;
        this.key = key;
        this.noteId = noteId;
    }
}