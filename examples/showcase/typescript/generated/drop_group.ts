import type { SoraReader } from "./sora_runtime.js";


export interface DropGroup {
    id: number;
    name: string;
}

export function decodeDropGroup(reader: SoraReader): DropGroup {
    return {
        id: reader.readI32(),
        name: reader.readString(),
    };
}
