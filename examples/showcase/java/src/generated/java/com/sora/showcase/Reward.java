package com.sora.showcase;

public final class Reward {
    public final Integer itemId;
    public final Integer count;

    public Reward(
        Integer itemId,
        Integer count
    ) {
        this.itemId = itemId;
        this.count = count;
    }

    static Reward decode(SoraReader reader) {
        return new Reward(
            reader.readI32(),
            reader.readI32()
        );
    }
}
