import type { SoraReader } from "./sora_runtime.js";


export interface Dialogue {
    id: number;
    speakerKey: string;
    lines: string[];
}

export function decodeDialogue(reader: SoraReader): Dialogue {
    return {
        id: reader.readI32(),
        speakerKey: reader.readString(),
        lines: reader.readList(() => reader.readString()),
    };
}
