import type { SoraReader } from "./sora_runtime.js";

import type { ElementType } from "./element_type.js";
import { decodeElementType } from "./element_type.js";

import type { ResourceCost } from "./resource_cost.js";
import { decodeResourceCost } from "./resource_cost.js";

import type { SkillEffect } from "./skill_effect.js";
import { decodeSkillEffect } from "./skill_effect.js";

import type { Vec3 } from "./vec3.js";
import { decodeVec3 } from "./vec3.js";


export interface Skill {
    id: number;
    name: string;
    element: ElementType;
    cost: ResourceCost;
    effect: SkillEffect;
    requiredLevel: number;
    requiredItem: number | undefined;
    castOrigin: Vec3;
}

export function decodeSkill(reader: SoraReader): Skill {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        element: decodeElementType(reader),
        cost: decodeResourceCost(reader),
        effect: decodeSkillEffect(reader),
        requiredLevel: reader.readI32(),
        requiredItem: reader.readOptional(() => reader.readI32()),
        castOrigin: decodeVec3(reader),
    };
}
