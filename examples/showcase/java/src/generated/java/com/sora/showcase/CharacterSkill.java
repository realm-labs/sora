package com.sora.showcase;

public final class CharacterSkill {
    public final Integer characterId;
    public final Integer skillId;
    public final Integer unlockLevel;

    public CharacterSkill(
        Integer characterId,
        Integer skillId,
        Integer unlockLevel
    ) {
        this.characterId = characterId;
        this.skillId = skillId;
        this.unlockLevel = unlockLevel;
    }

    static CharacterSkill decode(SoraReader reader) {
        return new CharacterSkill(
            reader.readI32(),
            reader.readI32(),
            reader.readI32()
        );
    }
}
