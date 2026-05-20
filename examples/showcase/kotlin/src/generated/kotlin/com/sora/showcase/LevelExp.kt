package com.sora.showcase

data class LevelExp(
    val level: Int,
    val exp: Long,
    val unlockFeature: String?,
) {
    companion object {
        fun decode(reader: SoraReader): LevelExp =
            LevelExp(
                level = reader.readI32(),
                exp = reader.readI64(),
                unlockFeature = reader.readOptional { reader.readString() },
            )
    }
}