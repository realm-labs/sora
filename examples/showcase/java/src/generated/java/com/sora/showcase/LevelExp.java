package com.sora.showcase;

public final class LevelExp {
    public final Integer level;
    public final Long exp;
    public final String unlockFeature;

    public LevelExp(
        Integer level,
        Long exp,
        String unlockFeature
    ) {
        this.level = level;
        this.exp = exp;
        this.unlockFeature = unlockFeature;
    }

    static LevelExp decode(SoraReader reader) {
        return new LevelExp(
            reader.readI32(),
            reader.readI64(),
            reader.readOptional(() -> reader.readString())
        );
    }
}
