

export function decodeCharacterSkill(reader) {
    return {
        characterId: reader.readI32(),
        skillId: reader.readI32(),
        unlockLevel: reader.readI32(),
    };
}
