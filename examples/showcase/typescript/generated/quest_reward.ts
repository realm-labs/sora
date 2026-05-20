import type { SoraReader } from "./sora_runtime.js";


export interface QuestReward {
    questId: number;
    seq: number;
    itemId: number;
    count: number;
}

export function decodeQuestReward(reader: SoraReader): QuestReward {
    return {
        questId: reader.readI32(),
        seq: reader.readI32(),
        itemId: reader.readI32(),
        count: reader.readI32(),
    };
}
