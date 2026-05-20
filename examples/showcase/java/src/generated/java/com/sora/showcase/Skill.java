package com.sora.showcase;

public final class Skill {
    public final Integer id;
    public final String name;
    public final ElementType element;
    public final ResourceCost cost;
    public final SkillEffect effect;
    public final Integer requiredLevel;
    public final Integer requiredItem;
    public final Vec3 castOrigin;

    public Skill(
        Integer id,
        String name,
        ElementType element,
        ResourceCost cost,
        SkillEffect effect,
        Integer requiredLevel,
        Integer requiredItem,
        Vec3 castOrigin
    ) {
        this.id = id;
        this.name = name;
        this.element = element;
        this.cost = cost;
        this.effect = effect;
        this.requiredLevel = requiredLevel;
        this.requiredItem = requiredItem;
        this.castOrigin = castOrigin;
    }

    static Skill decode(SoraReader reader) {
        return new Skill(
            reader.readI32(),
            reader.readString(),
            ElementType.decode(reader),
            ResourceCost.decode(reader),
            SkillEffect.decode(reader),
            reader.readI32(),
            reader.readOptional(() -> reader.readI32()),
            Vec3.decode(reader)
        );
    }
}
