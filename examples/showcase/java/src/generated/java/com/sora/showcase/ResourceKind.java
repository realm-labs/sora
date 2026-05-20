package com.sora.showcase;

public enum ResourceKind {
    Item,
    Gold,
    Diamond;

    static ResourceKind decode(SoraReader reader) {
        switch (reader.readU32()) {
            case 0:
                return Item;
            case 1:
                return Gold;
            case 2:
                return Diamond;
            default:
                throw new SoraReadException("invalid enum ordinal for ResourceKind");
        }
    }
}