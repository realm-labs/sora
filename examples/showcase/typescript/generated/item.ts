import type { SoraReader } from "./sora_runtime.js";

import type { ItemType } from "./item_type.js";
import { decodeItemType } from "./item_type.js";

import type { ResourceCost } from "./resource_cost.js";
import { decodeResourceCost } from "./resource_cost.js";


export interface Item {
    id: number;
    name: string;
    itemType: ItemType;
    maxStack: number;
    price: ResourceCost;
    tags: string[];
}

export function decodeItem(reader: SoraReader): Item {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        itemType: decodeItemType(reader),
        maxStack: reader.readI32(),
        price: decodeResourceCost(reader),
        tags: reader.readList(() => reader.readString()),
    };
}
