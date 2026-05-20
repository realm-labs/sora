

export function decodeDialogue(reader) {
    return {
        id: reader.readI32(),
        speakerKey: reader.readString(),
        lines: reader.readList(() => reader.readString()),
    };
}
