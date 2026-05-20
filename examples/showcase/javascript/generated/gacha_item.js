
import { decodeRarity } from "./rarity.js";

export function decodeGachaItem(reader) {
    return {
        poolId: reader.readI32(),
        itemId: reader.readI32(),
        rarity: decodeRarity(reader),
        weight: reader.readF32(),
    };
}
