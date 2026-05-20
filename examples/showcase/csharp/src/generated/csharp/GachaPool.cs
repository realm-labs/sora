#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record GachaPool(
    int Id,
    string Name,
    ResourceCost Cost
)
{
    internal static GachaPool Decode(SoraReader reader)
    {
        return new GachaPool(
            reader.ReadInt32(),
            reader.ReadString(),
            ResourceCost.Decode(reader)
        );
    }
}