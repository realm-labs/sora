package game_config_showcase

sealed class EventCondition {
    data class LevelAtLeast(
        val level: Int,
    ) : EventCondition()
    data class QuestCompleted(
        val questId: Int,
    ) : EventCondition()
    data class HasItem(
        val itemId: Int,
        val count: Int,
    ) : EventCondition()

    companion object {
        fun decode(reader: SoraReader): EventCondition =
            when (val ordinal = reader.readU32()) {
                0 -> LevelAtLeast(
                    level = reader.readI32(),
                )
                1 -> QuestCompleted(
                    questId = reader.readI32(),
                )
                2 -> HasItem(
                    itemId = reader.readI32(),
                    count = reader.readI32(),
                )
                else -> throw SoraReadException("invalid union ordinal $ordinal for EventCondition")
            }
    }
}