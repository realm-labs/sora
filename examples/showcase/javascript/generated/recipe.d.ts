import type { SoraReader } from "./sora_runtime.js";

import type { ResourceCost } from "./resource_cost.js";


export interface Recipe {
    id: number;
    resultItem: number;
    materials: ResourceCost[];
}

export declare function decodeRecipe(reader: SoraReader): Recipe;
