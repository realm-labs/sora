package com.sora.showcase

data class StatModifier(
    val stat: StatType,
    val value: Float,
    val isPercent: Boolean,
) {
    companion object {
        fun decode(reader: SoraReader): StatModifier =
            StatModifier(
                stat = StatType.decode(reader),
                value = reader.readF32(),
                isPercent = reader.readBool(),
            )
    }
}