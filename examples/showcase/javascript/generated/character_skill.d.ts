import type { SoraReader } from "./sora_runtime.js";


export interface CharacterSkill {
    characterId: number;
    skillId: number;
    unlockLevel: number;
}

export declare function decodeCharacterSkill(reader: SoraReader): CharacterSkill;
