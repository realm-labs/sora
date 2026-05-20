
import { decodeRarity } from "./rarity.js";
import { decodeVec3 } from "./vec3.js";

export function decodeCharacter(reader) {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        rarity: decodeRarity(reader),
        baseLevel: reader.readI32(),
        baseSkill: reader.readI32(),
        starterItems: reader.readList(() => reader.readI32()),
        spawnPos: decodeVec3(reader),
    };
}
