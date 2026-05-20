package com.sora.showcase

enum class StatType {
    Hp,
    Attack,
    Defense,
    Speed,
    CritRate;

    companion object {
        fun decode(reader: SoraReader): StatType =
            when (val ordinal = reader.readU32()) {
                0 -> Hp
                1 -> Attack
                2 -> Defense
                3 -> Speed
                4 -> CritRate
                else -> throw SoraReadException("invalid enum ordinal $ordinal for StatType")
            }
    }
}
