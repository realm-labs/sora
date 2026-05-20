#nullable enable

namespace com.sora.showcase;

public enum ResourceKind
{
    Item,
    Gold,
    Diamond,
}

internal static class ResourceKindCodec
{
    internal static ResourceKind Decode(SoraReader reader)
    {
        return reader.ReadUInt32() switch
        {
            0 => ResourceKind.Item,
            1 => ResourceKind.Gold,
            2 => ResourceKind.Diamond,
            var value => throw new SoraReadException($"invalid enum ordinal {value} for ResourceKind"),
        };
    }
}