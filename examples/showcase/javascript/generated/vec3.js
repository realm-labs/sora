

export function decodeVec3(reader) {
    return {
        x: reader.readF32(),
        y: reader.readF32(),
        z: reader.readF32(),
    };
}
