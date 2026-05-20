package com.sora.showcase

data class CharacterSkill(
    val characterId: Int,
    val skillId: Int,
    val unlockLevel: Int,
) {
    companion object {
        fun decode(reader: SoraReader): CharacterSkill =
            CharacterSkill(
                characterId = reader.readI32(),
                skillId = reader.readI32(),
                unlockLevel = reader.readI32(),
            )
    }
}
