package game_config_showcase

data class Buff(
    val id: Int,
    val name: String,
    val duration: Float,
    val modifiers: List<StatModifier>,
) {
    companion object {
        fun decode(reader: SoraReader): Buff =
            Buff(
                id = reader.readI32(),
                name = reader.readString(),
                duration = reader.readF32(),
                modifiers = reader.readList { StatModifier.decode(reader) },
            )
    }
}