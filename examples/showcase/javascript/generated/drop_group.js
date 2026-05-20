

export function decodeDropGroup(reader) {
    return {
        id: reader.readI32(),
        name: reader.readString(),
    };
}
