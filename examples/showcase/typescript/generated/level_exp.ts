import type { SoraReader } from "./sora_runtime.js";


export interface LevelExp {
    level: number;
    exp: bigint;
    unlockFeature: string | undefined;
}

export function decodeLevelExp(reader: SoraReader): LevelExp {
    return {
        level: reader.readI32(),
        exp: reader.readI64(),
        unlockFeature: reader.readOptional(() => reader.readString()),
    };
}
