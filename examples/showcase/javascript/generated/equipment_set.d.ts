import type { SoraReader } from "./sora_runtime.js";

import type { SkillEffect } from "./skill_effect.js";


export interface EquipmentSet {
    id: number;
    name: string;
    itemIds: number[];
    bonusEffect: SkillEffect;
}

export declare function decodeEquipmentSet(reader: SoraReader): EquipmentSet;
