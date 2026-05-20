import type { SoraReader } from "./sora_runtime.js";


export interface Reward {
    itemId: number;
    count: number;
}

export declare function decodeReward(reader: SoraReader): Reward;
