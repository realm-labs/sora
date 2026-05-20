package game_config_showcase

data class Localization(
    val key: String,
    val zhCn: String,
    val enUs: String,
    val note: String?,
) {
    companion object {
        fun decode(reader: SoraReader): Localization =
            Localization(
                key = reader.readString(),
                zhCn = reader.readString(),
                enUs = reader.readString(),
                note = reader.readOptional { reader.readString() },
            )
    }
}