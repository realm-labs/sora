import type { SoraReader } from "./sora_runtime.js";

import type { Rarity } from "./rarity.js";
import { decodeRarity } from "./rarity.js";


export interface GachaItem {
    poolId: number;
    itemId: number;
    rarity: Rarity;
    weight: number;
}

export function decodeGachaItem(reader: SoraReader): GachaItem {
    return {
        poolId: reader.readI32(),
        itemId: reader.readI32(),
        rarity: decodeRarity(reader),
        weight: reader.readF32(),
    };
}
