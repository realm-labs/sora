

export function decodeQuestReward(reader) {
    return {
        questId: reader.readI32(),
        seq: reader.readI32(),
        itemId: reader.readI32(),
        count: reader.readI32(),
    };
}
