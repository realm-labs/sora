package com.sora.showcase

data class MailReward(
    val mailId: Int,
    val seq: Int,
    val itemId: Int,
    val count: Int,
) {
    companion object {
        fun decode(reader: SoraReader): MailReward =
            MailReward(
                mailId = reader.readI32(),
                seq = reader.readI32(),
                itemId = reader.readI32(),
                count = reader.readI32(),
            )
    }
}