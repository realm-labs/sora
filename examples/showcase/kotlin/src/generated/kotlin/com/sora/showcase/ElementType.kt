package com.sora.showcase

enum class ElementType {
    Fire,
    Ice,
    Lightning,
    Physical;

    companion object {
        fun decode(reader: SoraReader): ElementType =
            when (val ordinal = reader.readU32()) {
                0 -> Fire
                1 -> Ice
                2 -> Lightning
                3 -> Physical
                else -> throw SoraReadException("invalid enum ordinal $ordinal for ElementType")
            }
    }
}