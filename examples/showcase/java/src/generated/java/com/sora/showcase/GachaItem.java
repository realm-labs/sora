package com.sora.showcase;

public final class GachaItem {
    public final Integer poolId;
    public final Integer itemId;
    public final Rarity rarity;
    public final Float weight;

    public GachaItem(
        Integer poolId,
        Integer itemId,
        Rarity rarity,
        Float weight
    ) {
        this.poolId = poolId;
        this.itemId = itemId;
        this.rarity = rarity;
        this.weight = weight;
    }

    static GachaItem decode(SoraReader reader) {
        return new GachaItem(
            reader.readI32(),
            reader.readI32(),
            Rarity.decode(reader),
            reader.readF32()
        );
    }
}