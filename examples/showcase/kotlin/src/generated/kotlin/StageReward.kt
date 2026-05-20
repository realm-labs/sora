package game_config_showcase

data class StageReward(
    val stageId: Int,
    val seq: Int,
    val itemId: Int,
    val count: Int,
) {
    companion object {
        fun decode(reader: SoraReader): StageReward =
            StageReward(
                stageId = reader.readI32(),
                seq = reader.readI32(),
                itemId = reader.readI32(),
                count = reader.readI32(),
            )
    }
}