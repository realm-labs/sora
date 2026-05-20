package com.sora.showcase;

public final class EquipmentSet {
    public final Integer id;
    public final String name;
    public final java.util.List<Integer> itemIds;
    public final SkillEffect bonusEffect;

    public EquipmentSet(
        Integer id,
        String name,
        java.util.List<Integer> itemIds,
        SkillEffect bonusEffect
    ) {
        this.id = id;
        this.name = name;
        this.itemIds = itemIds;
        this.bonusEffect = bonusEffect;
    }

    static EquipmentSet decode(SoraReader reader) {
        return new EquipmentSet(
            reader.readI32(),
            reader.readString(),
            reader.readList(() -> reader.readI32()),
            SkillEffect.decode(reader)
        );
    }
}