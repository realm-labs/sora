package com.sora.showcase;

public final class Item {
    public final Integer id;
    public final String name;
    public final ItemType itemType;
    public final Integer maxStack;
    public final ResourceCost price;
    public final java.util.List<String> tags;

    public Item(
        Integer id,
        String name,
        ItemType itemType,
        Integer maxStack,
        ResourceCost price,
        java.util.List<String> tags
    ) {
        this.id = id;
        this.name = name;
        this.itemType = itemType;
        this.maxStack = maxStack;
        this.price = price;
        this.tags = tags;
    }

    static Item decode(SoraReader reader) {
        return new Item(
            reader.readI32(),
            reader.readString(),
            ItemType.decode(reader),
            reader.readI32(),
            ResourceCost.decode(reader),
            reader.readList(() -> reader.readString())
        );
    }
}