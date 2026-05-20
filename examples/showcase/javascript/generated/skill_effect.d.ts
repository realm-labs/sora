import type { SoraReader } from "./sora_runtime.js";

import type { ElementType } from "./element_type.js";


export interface SkillEffect {
    element: ElementType;
    power: number;
    radius: number;
}

export declare function decodeSkillEffect(reader: SoraReader): SkillEffect;
