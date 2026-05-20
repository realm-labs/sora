package com.sora.showcase;

public final class Monster {
    public final Integer id;
    public final String name;
    public final Integer level;
    public final ElementType element;
    public final Integer dropGroup;
    public final Vec3 spawnPos;

    public Monster(
        Integer id,
        String name,
        Integer level,
        ElementType element,
        Integer dropGroup,
        Vec3 spawnPos
    ) {
        this.id = id;
        this.name = name;
        this.level = level;
        this.element = element;
        this.dropGroup = dropGroup;
        this.spawnPos = spawnPos;
    }

    static Monster decode(SoraReader reader) {
        return new Monster(
            reader.readI32(),
            reader.readString(),
            reader.readI32(),
            ElementType.decode(reader),
            reader.readI32(),
            Vec3.decode(reader)
        );
    }
}