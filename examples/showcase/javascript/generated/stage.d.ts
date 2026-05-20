import type { SoraReader } from "./sora_runtime.js";

import type { Reward } from "./reward.js";


export interface Stage {
    id: number;
    name: string;
    monsterIds: number[];
    recommendedPower: number;
    firstClearRewards: Reward[];
}

export declare function decodeStage(reader: SoraReader): Stage;
