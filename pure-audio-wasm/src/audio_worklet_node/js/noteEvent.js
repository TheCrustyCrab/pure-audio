export class NoteEvent {
    constructor(eventType, portIndex, channel, key, noteId) {
        this.eventType = eventType;
        this.portIndex = portIndex;
        this.channel = channel;
        this.key = key;
        this.noteId = noteId;
    }
}