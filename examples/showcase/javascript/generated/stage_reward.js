

export function decodeStageReward(reader) {
    return {
        stageId: reader.readI32(),
        seq: reader.readI32(),
        itemId: reader.readI32(),
        count: reader.readI32(),
    };
}
