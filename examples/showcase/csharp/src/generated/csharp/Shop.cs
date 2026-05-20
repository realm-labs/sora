#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record Shop(
    int Id,
    string Name,
    ResourceKind Currency
)
{
    internal static Shop Decode(SoraReader reader)
    {
        return new Shop(
            reader.ReadInt32(),
            reader.ReadString(),
            ResourceKindCodec.Decode(reader)
        );
    }
}
