package game_config_showcase

enum class ItemType {
    Weapon,
    Armor,
    Currency,
    Material,
    Consumable;

    companion object {
        fun decode(reader: SoraReader): ItemType =
            when (val ordinal = reader.readU32()) {
                0 -> Weapon
                1 -> Armor
                2 -> Currency
                3 -> Material
                4 -> Consumable
                else -> throw SoraReadException("invalid enum ordinal $ordinal for ItemType")
            }
    }
}