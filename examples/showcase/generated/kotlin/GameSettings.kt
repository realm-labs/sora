package game_config_showcase

data class GameSettings(
    val version: String,
    val dailyResetHour: Int,
    val startingGold: Int,
    val spawnPos: Vec3,
    val starterItems: List<Int>,
) {
    companion object {
        fun decode(reader: SoraReader): GameSettings =
            GameSettings(
                version = reader.readString(),
                dailyResetHour = reader.readI32(),
                startingGold = reader.readI32(),
                spawnPos = Vec3.decode(reader),
                starterItems = reader.readList { reader.readI32() },
            )
    }
}