package game_config_showcase

data class QuestReward(
    val questId: Int,
    val seq: Int,
    val itemId: Int,
    val count: Int,
) {
    companion object {
        fun decode(reader: SoraReader): QuestReward =
            QuestReward(
                questId = reader.readI32(),
                seq = reader.readI32(),
                itemId = reader.readI32(),
                count = reader.readI32(),
            )
    }
}