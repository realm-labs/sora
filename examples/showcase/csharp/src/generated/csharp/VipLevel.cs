#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record VipLevel(
    int Level,
    ResourceCost Cost,
    List<string> Perks
)
{
    internal static VipLevel Decode(SoraReader reader)
    {
        return new VipLevel(
            reader.ReadInt32(),
            ResourceCost.Decode(reader),
            reader.ReadList(() => reader.ReadString())
        );
    }
}