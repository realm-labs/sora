package com.sora.showcase

enum class MailType {
    System,
    Event,
    Compensation;

    companion object {
        fun decode(reader: SoraReader): MailType =
            when (val ordinal = reader.readU32()) {
                0 -> System
                1 -> Event
                2 -> Compensation
                else -> throw SoraReadException("invalid enum ordinal $ordinal for MailType")
            }
    }
}
