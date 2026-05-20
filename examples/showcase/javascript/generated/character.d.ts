import type { SoraReader } from "./sora_runtime.js";

import type { Rarity } from "./rarity.js";

import type { Vec3 } from "./vec3.js";


export interface Character {
    id: number;
    name: string;
    rarity: Rarity;
    baseLevel: number;
    baseSkill: number;
    starterItems: number[];
    spawnPos: Vec3;
}

export declare function decodeCharacter(reader: SoraReader): Character;
