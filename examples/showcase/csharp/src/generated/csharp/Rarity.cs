#nullable enable

namespace com.sora.showcase;

public enum Rarity
{
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

internal static class RarityCodec
{
    internal static Rarity Decode(SoraReader reader)
    {
        return reader.ReadUInt32() switch
        {
            0 => Rarity.Common,
            1 => Rarity.Uncommon,
            2 => Rarity.Rare,
            3 => Rarity.Epic,
            4 => Rarity.Legendary,
            var value => throw new SoraReadException($"invalid enum ordinal {value} for Rarity"),
        };
    }
}
