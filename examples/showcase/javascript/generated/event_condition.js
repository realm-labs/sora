

export function decodeEventCondition(reader) {
    const ordinal = reader.readU32();
    if (ordinal === 0) {
        return {
            type: "LevelAtLeast",
            level: reader.readI32(),
        };
    }
    if (ordinal === 1) {
        return {
            type: "QuestCompleted",
            questId: reader.readI32(),
        };
    }
    if (ordinal === 2) {
        return {
            type: "HasItem",
            itemId: reader.readI32(),
            count: reader.readI32(),
        };
    }
    throw new Error(`invalid union ordinal ${ordinal} for EventCondition`);
}
