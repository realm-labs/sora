#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record GachaItem(
    int PoolId,
    int ItemId,
    Rarity Rarity,
    float Weight
)
{
    internal static GachaItem Decode(SoraReader reader)
    {
        return new GachaItem(
            reader.ReadInt32(),
            reader.ReadInt32(),
            RarityCodec.Decode(reader),
            reader.ReadFloat()
        );
    }
}