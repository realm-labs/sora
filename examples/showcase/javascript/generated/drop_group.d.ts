import type { SoraReader } from "./sora_runtime.js";


export interface DropGroup {
    id: number;
    name: string;
}

export declare function decodeDropGroup(reader: SoraReader): DropGroup;
