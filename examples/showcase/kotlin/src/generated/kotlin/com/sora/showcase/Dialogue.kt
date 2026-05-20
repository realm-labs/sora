package com.sora.showcase

data class Dialogue(
    val id: Int,
    val speakerKey: String,
    val lines: List<String>,
) {
    companion object {
        fun decode(reader: SoraReader): Dialogue =
            Dialogue(
                id = reader.readI32(),
                speakerKey = reader.readString(),
                lines = reader.readList { reader.readString() },
            )
    }
}
