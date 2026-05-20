

export function decodeRewardAction(reader) {
    const ordinal = reader.readU32();
    if (ordinal === 0) {
        return {
            type: "AddItem",
            itemId: reader.readI32(),
            count: reader.readI32(),
        };
    }
    if (ordinal === 1) {
        return {
            type: "AddBuff",
            buffId: reader.readI32(),
            duration: reader.readF32(),
        };
    }
    if (ordinal === 2) {
        return {
            type: "UnlockStage",
            stageId: reader.readI32(),
        };
    }
    if (ordinal === 3) {
        return {
            type: "SendMail",
            mailId: reader.readI32(),
        };
    }
    throw new Error(`invalid union ordinal ${ordinal} for RewardAction`);
}
