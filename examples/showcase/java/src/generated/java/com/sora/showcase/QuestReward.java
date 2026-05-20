package com.sora.showcase;

public final class QuestReward {
    public final Integer questId;
    public final Integer seq;
    public final Integer itemId;
    public final Integer count;

    public QuestReward(
        Integer questId,
        Integer seq,
        Integer itemId,
        Integer count
    ) {
        this.questId = questId;
        this.seq = seq;
        this.itemId = itemId;
        this.count = count;
    }

    static QuestReward decode(SoraReader reader) {
        return new QuestReward(
            reader.readI32(),
            reader.readI32(),
            reader.readI32(),
            reader.readI32()
        );
    }
}