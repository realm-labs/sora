package com.sora.showcase;

public final class Buff {
    public final Integer id;
    public final String name;
    public final Float duration;
    public final java.util.List<StatModifier> modifiers;

    public Buff(
        Integer id,
        String name,
        Float duration,
        java.util.List<StatModifier> modifiers
    ) {
        this.id = id;
        this.name = name;
        this.duration = duration;
        this.modifiers = modifiers;
    }

    static Buff decode(SoraReader reader) {
        return new Buff(
            reader.readI32(),
            reader.readString(),
            reader.readF32(),
            reader.readList(() -> StatModifier.decode(reader))
        );
    }
}
