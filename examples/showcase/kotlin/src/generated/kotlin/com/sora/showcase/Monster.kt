package com.sora.showcase

data class Monster(
    val id: Int,
    val name: String,
    val level: Int,
    val element: ElementType,
    val dropGroup: Int,
    val spawnPos: Vec3,
) {
    companion object {
        fun decode(reader: SoraReader): Monster =
            Monster(
                id = reader.readI32(),
                name = reader.readString(),
                level = reader.readI32(),
                element = ElementType.decode(reader),
                dropGroup = reader.readI32(),
                spawnPos = Vec3.decode(reader),
            )
    }
}
