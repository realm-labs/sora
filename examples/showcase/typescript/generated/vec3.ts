import type { SoraReader } from "./sora_runtime.js";


export interface Vec3 {
    x: number;
    y: number;
    z: number;
}

export function decodeVec3(reader: SoraReader): Vec3 {
    return {
        x: reader.readF32(),
        y: reader.readF32(),
        z: reader.readF32(),
    };
}
