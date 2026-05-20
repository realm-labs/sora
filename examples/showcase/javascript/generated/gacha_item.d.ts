import type { SoraReader } from "./sora_runtime.js";

import type { Rarity } from "./rarity.js";


export interface GachaItem {
    poolId: number;
    itemId: number;
    rarity: Rarity;
    weight: number;
}

export declare function decodeGachaItem(reader: SoraReader): GachaItem;
