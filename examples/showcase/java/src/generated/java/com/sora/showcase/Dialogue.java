package com.sora.showcase;

public final class Dialogue {
    public final Integer id;
    public final String speakerKey;
    public final java.util.List<String> lines;

    public Dialogue(
        Integer id,
        String speakerKey,
        java.util.List<String> lines
    ) {
        this.id = id;
        this.speakerKey = speakerKey;
        this.lines = lines;
    }

    static Dialogue decode(SoraReader reader) {
        return new Dialogue(
            reader.readI32(),
            reader.readString(),
            reader.readList(() -> reader.readString())
        );
    }
}