package com.sora.showcase;

public final class Character {
    public final Integer id;
    public final String name;
    public final Rarity rarity;
    public final Integer baseLevel;
    public final Integer baseSkill;
    public final java.util.List<Integer> starterItems;
    public final Vec3 spawnPos;

    public Character(
        Integer id,
        String name,
        Rarity rarity,
        Integer baseLevel,
        Integer baseSkill,
        java.util.List<Integer> starterItems,
        Vec3 spawnPos
    ) {
        this.id = id;
        this.name = name;
        this.rarity = rarity;
        this.baseLevel = baseLevel;
        this.baseSkill = baseSkill;
        this.starterItems = starterItems;
        this.spawnPos = spawnPos;
    }

    static Character decode(SoraReader reader) {
        return new Character(
            reader.readI32(),
            reader.readString(),
            Rarity.decode(reader),
            reader.readI32(),
            reader.readI32(),
            reader.readList(() -> reader.readI32()),
            Vec3.decode(reader)
        );
    }
}