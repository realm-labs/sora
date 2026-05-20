package com.sora.showcase;

public final class Localization {
    public final String key;
    public final String zhCn;
    public final String enUs;
    public final String note;

    public Localization(
        String key,
        String zhCn,
        String enUs,
        String note
    ) {
        this.key = key;
        this.zhCn = zhCn;
        this.enUs = enUs;
        this.note = note;
    }

    static Localization decode(SoraReader reader) {
        return new Localization(
            reader.readString(),
            reader.readString(),
            reader.readString(),
            reader.readOptional(() -> reader.readString())
        );
    }
}
