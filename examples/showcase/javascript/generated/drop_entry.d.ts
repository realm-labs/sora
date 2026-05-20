import type { SoraReader } from "./sora_runtime.js";


export interface DropEntry {
    groupId: number;
    seq: number;
    itemId: number;
    count: number;
    weight: number;
}

export declare function decodeDropEntry(reader: SoraReader): DropEntry;
