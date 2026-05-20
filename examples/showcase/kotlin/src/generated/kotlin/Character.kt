package game_config_showcase

data class Character(
    val id: Int,
    val name: String,
    val rarity: Rarity,
    val baseLevel: Int,
    val baseSkill: Int,
    val starterItems: List<Int>,
    val spawnPos: Vec3,
) {
    companion object {
        fun decode(reader: SoraReader): Character =
            Character(
                id = reader.readI32(),
                name = reader.readString(),
                rarity = Rarity.decode(reader),
                baseLevel = reader.readI32(),
                baseSkill = reader.readI32(),
                starterItems = reader.readList { reader.readI32() },
                spawnPos = Vec3.decode(reader),
            )
    }
}