import type { SoraReader } from "./sora_runtime.js";

import type { ResourceCost } from "./resource_cost.js";
import { decodeResourceCost } from "./resource_cost.js";


export interface Dungeon {
    id: number;
    name: string;
    stageIds: number[];
    entryCost: ResourceCost;
}

export function decodeDungeon(reader: SoraReader): Dungeon {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        stageIds: reader.readList(() => reader.readI32()),
        entryCost: decodeResourceCost(reader),
    };
}
