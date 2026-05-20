import type { SoraReader } from "./sora_runtime.js";

import type { ElementType } from "./element_type.js";

import type { Vec3 } from "./vec3.js";


export interface Monster {
    id: number;
    name: string;
    level: number;
    element: ElementType;
    dropGroup: number;
    spawnPos: Vec3;
}

export declare function decodeMonster(reader: SoraReader): Monster;
