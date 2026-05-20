package com.sora.showcase;

public final class Vec3 {
    public final Float x;
    public final Float y;
    public final Float z;

    public Vec3(
        Float x,
        Float y,
        Float z
    ) {
        this.x = x;
        this.y = y;
        this.z = z;
    }

    static Vec3 decode(SoraReader reader) {
        return new Vec3(
            reader.readF32(),
            reader.readF32(),
            reader.readF32()
        );
    }
}