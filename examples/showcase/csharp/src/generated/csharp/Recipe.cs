#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record Recipe(
    int Id,
    int ResultItem,
    List<ResourceCost> Materials
)
{
    internal static Recipe Decode(SoraReader reader)
    {
        return new Recipe(
            reader.ReadInt32(),
            reader.ReadInt32(),
            reader.ReadList(() => ResourceCost.Decode(reader))
        );
    }
}
