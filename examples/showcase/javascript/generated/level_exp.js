

export function decodeLevelExp(reader) {
    return {
        level: reader.readI32(),
        exp: reader.readI64(),
        unlockFeature: reader.readOptional(() => reader.readString()),
    };
}
