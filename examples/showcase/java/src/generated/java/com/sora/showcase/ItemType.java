package com.sora.showcase;

public enum ItemType {
    Weapon,
    Armor,
    Currency,
    Material,
    Consumable;

    static ItemType decode(SoraReader reader) {
        switch (reader.readU32()) {
            case 0:
                return Weapon;
            case 1:
                return Armor;
            case 2:
                return Currency;
            case 3:
                return Material;
            case 4:
                return Consumable;
            default:
                throw new SoraReadException("invalid enum ordinal for ItemType");
        }
    }
}