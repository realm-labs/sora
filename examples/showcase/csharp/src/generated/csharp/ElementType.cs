#nullable enable

namespace com.sora.showcase;

public enum ElementType
{
    Fire,
    Ice,
    Lightning,
    Physical,
}

internal static class ElementTypeCodec
{
    internal static ElementType Decode(SoraReader reader)
    {
        return reader.ReadUInt32() switch
        {
            0 => ElementType.Fire,
            1 => ElementType.Ice,
            2 => ElementType.Lightning,
            3 => ElementType.Physical,
            var value => throw new SoraReadException($"invalid enum ordinal {value} for ElementType"),
        };
    }
}
