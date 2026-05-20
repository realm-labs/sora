import type { SoraReader } from "./sora_runtime.js";

import type { StatType } from "./stat_type.js";


export interface StatModifier {
    stat: StatType;
    value: number;
    isPercent: boolean;
}

export declare function decodeStatModifier(reader: SoraReader): StatModifier;
