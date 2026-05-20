

export function decodeReward(reader) {
    return {
        itemId: reader.readI32(),
        count: reader.readI32(),
    };
}
