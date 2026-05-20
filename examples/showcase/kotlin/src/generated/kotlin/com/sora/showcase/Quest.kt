package com.sora.showcase

data class Quest(
    val id: Int,
    val questType: QuestType,
    val title: String,
    val requiredItem: Int,
    val unlockSkills: List<Int>,
    val startPos: Vec3,
    val rewards: List<Reward>,
) {
    companion object {
        fun decode(reader: SoraReader): Quest =
            Quest(
                id = reader.readI32(),
                questType = QuestType.decode(reader),
                title = reader.readString(),
                requiredItem = reader.readI32(),
                unlockSkills = reader.readList { reader.readI32() },
                startPos = Vec3.decode(reader),
                rewards = reader.readList { Reward.decode(reader) },
            )
    }
}
