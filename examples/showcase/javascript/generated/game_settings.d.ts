import type { SoraReader } from "./sora_runtime.js";

import type { Vec3 } from "./vec3.js";


export interface GameSettings {
    version: string;
    dailyResetHour: number;
    startingGold: number;
    spawnPos: Vec3;
    starterItems: number[];
}

export declare function decodeGameSettings(reader: SoraReader): GameSettings;
