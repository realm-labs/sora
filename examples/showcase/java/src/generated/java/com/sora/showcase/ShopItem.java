package com.sora.showcase;

public final class ShopItem {
    public final Integer shopId;
    public final Integer seq;
    public final Integer itemId;
    public final ResourceCost price;
    public final Integer dailyLimit;

    public ShopItem(
        Integer shopId,
        Integer seq,
        Integer itemId,
        ResourceCost price,
        Integer dailyLimit
    ) {
        this.shopId = shopId;
        this.seq = seq;
        this.itemId = itemId;
        this.price = price;
        this.dailyLimit = dailyLimit;
    }

    static ShopItem decode(SoraReader reader) {
        return new ShopItem(
            reader.readI32(),
            reader.readI32(),
            reader.readI32(),
            ResourceCost.decode(reader),
            reader.readOptional(() -> reader.readI32())
        );
    }
}
