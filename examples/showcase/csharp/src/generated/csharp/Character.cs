#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record Character(
    int Id,
    string Name,
    Rarity Rarity,
    int BaseLevel,
    int BaseSkill,
    List<int> StarterItems,
    Vec3 SpawnPos
)
{
    internal static Character Decode(SoraReader reader)
    {
        return new Character(
            reader.ReadInt32(),
            reader.ReadString(),
            RarityCodec.Decode(reader),
            reader.ReadInt32(),
            reader.ReadInt32(),
            reader.ReadList(() => reader.ReadInt32()),
            Vec3.Decode(reader)
        );
    }
}
