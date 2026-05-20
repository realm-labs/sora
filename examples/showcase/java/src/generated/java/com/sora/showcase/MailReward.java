package com.sora.showcase;

public final class MailReward {
    public final Integer mailId;
    public final Integer seq;
    public final Integer itemId;
    public final Integer count;

    public MailReward(
        Integer mailId,
        Integer seq,
        Integer itemId,
        Integer count
    ) {
        this.mailId = mailId;
        this.seq = seq;
        this.itemId = itemId;
        this.count = count;
    }

    static MailReward decode(SoraReader reader) {
        return new MailReward(
            reader.readI32(),
            reader.readI32(),
            reader.readI32(),
            reader.readI32()
        );
    }
}
