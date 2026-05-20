import type { SoraReader } from "./sora_runtime.js";

import type { StatModifier } from "./stat_modifier.js";


export interface Buff {
    id: number;
    name: string;
    duration: number;
    modifiers: StatModifier[];
}

export declare function decodeBuff(reader: SoraReader): Buff;
