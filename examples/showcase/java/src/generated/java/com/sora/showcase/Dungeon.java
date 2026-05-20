package com.sora.showcase;

public final class Dungeon {
    public final Integer id;
    public final String name;
    public final java.util.List<Integer> stageIds;
    public final ResourceCost entryCost;

    public Dungeon(
        Integer id,
        String name,
        java.util.List<Integer> stageIds,
        ResourceCost entryCost
    ) {
        this.id = id;
        this.name = name;
        this.stageIds = stageIds;
        this.entryCost = entryCost;
    }

    static Dungeon decode(SoraReader reader) {
        return new Dungeon(
            reader.readI32(),
            reader.readString(),
            reader.readList(() -> reader.readI32()),
            ResourceCost.decode(reader)
        );
    }
}