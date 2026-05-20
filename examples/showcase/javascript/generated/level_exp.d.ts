import type { SoraReader } from "./sora_runtime.js";


export interface LevelExp {
    level: number;
    exp: bigint;
    unlockFeature: string | undefined;
}

export declare function decodeLevelExp(reader: SoraReader): LevelExp;
