package game_config_showcase

data class DropGroup(
    val id: Int,
    val name: String,
) {
    companion object {
        fun decode(reader: SoraReader): DropGroup =
            DropGroup(
                id = reader.readI32(),
                name = reader.readString(),
            )
    }
}