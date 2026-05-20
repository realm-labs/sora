import type { SoraReader } from "./sora_runtime.js";

import type { ResourceCost } from "./resource_cost.js";


export interface GachaPool {
    id: number;
    name: string;
    cost: ResourceCost;
}

export declare function decodeGachaPool(reader: SoraReader): GachaPool;
