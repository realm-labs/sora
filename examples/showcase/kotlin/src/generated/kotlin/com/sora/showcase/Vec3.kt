package com.sora.showcase

data class Vec3(
    val x: Float,
    val y: Float,
    val z: Float,
) {
    companion object {
        fun decode(reader: SoraReader): Vec3 =
            Vec3(
                x = reader.readF32(),
                y = reader.readF32(),
                z = reader.readF32(),
            )
    }
}
