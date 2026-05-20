import type { SoraReader } from "./sora_runtime.js";


export interface Dialogue {
    id: number;
    speakerKey: string;
    lines: string[];
}

export declare function decodeDialogue(reader: SoraReader): Dialogue;
