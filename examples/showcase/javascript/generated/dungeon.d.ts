import type { SoraReader } from "./sora_runtime.js";

import type { ResourceCost } from "./resource_cost.js";


export interface Dungeon {
    id: number;
    name: string;
    stageIds: number[];
    entryCost: ResourceCost;
}

export declare function decodeDungeon(reader: SoraReader): Dungeon;
