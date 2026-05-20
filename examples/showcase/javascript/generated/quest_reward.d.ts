import type { SoraReader } from "./sora_runtime.js";


export interface QuestReward {
    questId: number;
    seq: number;
    itemId: number;
    count: number;
}

export declare function decodeQuestReward(reader: SoraReader): QuestReward;
