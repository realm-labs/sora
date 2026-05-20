import type { SoraReader } from "./sora_runtime.js";

import type { Rarity } from "./rarity.js";
import { decodeRarity } from "./rarity.js";

import type { Vec3 } from "./vec3.js";
import { decodeVec3 } from "./vec3.js";


export interface Character {
    id: number;
    name: string;
    rarity: Rarity;
    baseLevel: number;
    baseSkill: number;
    starterItems: number[];
    spawnPos: Vec3;
}

export function decodeCharacter(reader: SoraReader): Character {
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
