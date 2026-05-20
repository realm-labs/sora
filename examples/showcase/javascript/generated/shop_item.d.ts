import type { SoraReader } from "./sora_runtime.js";

import type { ResourceCost } from "./resource_cost.js";


export interface ShopItem {
    shopId: number;
    seq: number;
    itemId: number;
    price: ResourceCost;
    dailyLimit: number | undefined;
}

export declare function decodeShopItem(reader: SoraReader): ShopItem;
