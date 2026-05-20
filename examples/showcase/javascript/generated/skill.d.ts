import type { SoraReader } from "./sora_runtime.js";

import type { ElementType } from "./element_type.js";

import type { ResourceCost } from "./resource_cost.js";

import type { SkillEffect } from "./skill_effect.js";

import type { Vec3 } from "./vec3.js";


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

export declare function decodeSkill(reader: SoraReader): Skill;
