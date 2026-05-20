package com.sora.showcase;

public final class Shop {
    public final Integer id;
    public final String name;
    public final ResourceKind currency;

    public Shop(
        Integer id,
        String name,
        ResourceKind currency
    ) {
        this.id = id;
        this.name = name;
        this.currency = currency;
    }

    static Shop decode(SoraReader reader) {
        return new Shop(
            reader.readI32(),
            reader.readString(),
            ResourceKind.decode(reader)
        );
    }
}
