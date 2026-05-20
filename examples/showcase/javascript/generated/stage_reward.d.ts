import type { SoraReader } from "./sora_runtime.js";


export interface StageReward {
    stageId: number;
    seq: number;
    itemId: number;
    count: number;
}

export declare function decodeStageReward(reader: SoraReader): StageReward;
