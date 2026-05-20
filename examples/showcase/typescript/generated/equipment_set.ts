import type { SoraReader } from "./sora_runtime.js";

import type { SkillEffect } from "./skill_effect.js";
import { decodeSkillEffect } from "./skill_effect.js";


export interface EquipmentSet {
    id: number;
    name: string;
    itemIds: number[];
    bonusEffect: SkillEffect;
}

export function decodeEquipmentSet(reader: SoraReader): EquipmentSet {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        itemIds: reader.readList(() => reader.readI32()),
        bonusEffect: decodeSkillEffect(reader),
    };
}
