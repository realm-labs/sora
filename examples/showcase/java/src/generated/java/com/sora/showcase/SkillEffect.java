package com.sora.showcase;

public final class SkillEffect {
    public final ElementType element;
    public final Integer power;
    public final Float radius;

    public SkillEffect(
        ElementType element,
        Integer power,
        Float radius
    ) {
        this.element = element;
        this.power = power;
        this.radius = radius;
    }

    static SkillEffect decode(SoraReader reader) {
        return new SkillEffect(
            ElementType.decode(reader),
            reader.readI32(),
            reader.readF32()
        );
    }
}
