package com.sora.showcase;

public final class StatModifier {
    public final StatType stat;
    public final Float value;
    public final Boolean isPercent;

    public StatModifier(
        StatType stat,
        Float value,
        Boolean isPercent
    ) {
        this.stat = stat;
        this.value = value;
        this.isPercent = isPercent;
    }

    static StatModifier decode(SoraReader reader) {
        return new StatModifier(
            StatType.decode(reader),
            reader.readF32(),
            reader.readBool()
        );
    }
}
