#nullable enable

namespace com.sora.showcase;

public enum StatType
{
    Hp,
    Attack,
    Defense,
    Speed,
    CritRate,
}

internal static class StatTypeCodec
{
    internal static StatType Decode(SoraReader reader)
    {
        return reader.ReadUInt32() switch
        {
            0 => StatType.Hp,
            1 => StatType.Attack,
            2 => StatType.Defense,
            3 => StatType.Speed,
            4 => StatType.CritRate,
            var value => throw new SoraReadException($"invalid enum ordinal {value} for StatType"),
        };
    }
}