package com.sora.showcase

data class DropEntry(
    val groupId: Int,
    val seq: Int,
    val itemId: Int,
    val count: Int,
    val weight: Float,
) {
    companion object {
        fun decode(reader: SoraReader): DropEntry =
            DropEntry(
                groupId = reader.readI32(),
                seq = reader.readI32(),
                itemId = reader.readI32(),
                count = reader.readI32(),
                weight = reader.readF32(),
            )
    }
}