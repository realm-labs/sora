#nullable enable

namespace com.sora.showcase;

public enum ItemType
{
    Weapon,
    Armor,
    Currency,
    Material,
    Consumable,
}

internal static class ItemTypeCodec
{
    internal static ItemType Decode(SoraReader reader)
    {
        return reader.ReadUInt32() switch
        {
            0 => ItemType.Weapon,
            1 => ItemType.Armor,
            2 => ItemType.Currency,
            3 => ItemType.Material,
            4 => ItemType.Consumable,
            var value => throw new SoraReadException($"invalid enum ordinal {value} for ItemType"),
        };
    }
}