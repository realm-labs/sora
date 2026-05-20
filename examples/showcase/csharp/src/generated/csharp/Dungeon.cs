#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record Dungeon(
    int Id,
    string Name,
    List<int> StageIds,
    ResourceCost EntryCost
)
{
    internal static Dungeon Decode(SoraReader reader)
    {
        return new Dungeon(
            reader.ReadInt32(),
            reader.ReadString(),
            reader.ReadList(() => reader.ReadInt32()),
            ResourceCost.Decode(reader)
        );
    }
}