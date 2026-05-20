package com.sora.showcase;

public final class VipLevel {
    public final Integer level;
    public final ResourceCost cost;
    public final java.util.List<String> perks;

    public VipLevel(
        Integer level,
        ResourceCost cost,
        java.util.List<String> perks
    ) {
        this.level = level;
        this.cost = cost;
        this.perks = perks;
    }

    static VipLevel decode(SoraReader reader) {
        return new VipLevel(
            reader.readI32(),
            ResourceCost.decode(reader),
            reader.readList(() -> reader.readString())
        );
    }
}