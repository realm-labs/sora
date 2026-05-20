package com.sora.showcase;

public final class DropGroup {
    public final Integer id;
    public final String name;

    public DropGroup(
        Integer id,
        String name
    ) {
        this.id = id;
        this.name = name;
    }

    static DropGroup decode(SoraReader reader) {
        return new DropGroup(
            reader.readI32(),
            reader.readString()
        );
    }
}