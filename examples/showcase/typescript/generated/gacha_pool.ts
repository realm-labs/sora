import type { SoraReader } from "./sora_runtime.js";

import type { ResourceCost } from "./resource_cost.js";
import { decodeResourceCost } from "./resource_cost.js";


export interface GachaPool {
    id: number;
    name: string;
    cost: ResourceCost;
}

export function decodeGachaPool(reader: SoraReader): GachaPool {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        cost: decodeResourceCost(reader),
    };
}
