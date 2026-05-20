import type { SoraReader } from "./sora_runtime.js";

import type { ResourceCost } from "./resource_cost.js";


export interface Achievement {
    id: number;
    titleKey: string;
    targetCount: bigint;
    reward: ResourceCost;
}

export declare function decodeAchievement(reader: SoraReader): Achievement;
