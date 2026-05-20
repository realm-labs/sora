import type { SoraReader } from "./sora_runtime.js";


export interface CharacterSkill {
    characterId: number;
    skillId: number;
    unlockLevel: number;
}

export function decodeCharacterSkill(reader: SoraReader): CharacterSkill {
    return {
        characterId: reader.readI32(),
        skillId: reader.readI32(),
        unlockLevel: reader.readI32(),
    };
}
