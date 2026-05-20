package com.sora.showcase;

public final class Recipe {
    public final Integer id;
    public final Integer resultItem;
    public final java.util.List<ResourceCost> materials;

    public Recipe(
        Integer id,
        Integer resultItem,
        java.util.List<ResourceCost> materials
    ) {
        this.id = id;
        this.resultItem = resultItem;
        this.materials = materials;
    }

    static Recipe decode(SoraReader reader) {
        return new Recipe(
            reader.readI32(),
            reader.readI32(),
            reader.readList(() -> ResourceCost.decode(reader))
        );
    }
}