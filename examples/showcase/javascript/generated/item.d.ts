import type { SoraReader } from "./sora_runtime.js";

import type { ItemType } from "./item_type.js";

import type { ResourceCost } from "./resource_cost.js";


export interface Item {
    id: number;
    name: string;
    itemType: ItemType;
    maxStack: number;
    price: ResourceCost;
    tags: string[];
}

export declare function decodeItem(reader: SoraReader): Item;
