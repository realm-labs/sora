#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record Item(
    int Id,
    string Name,
    ItemType ItemType,
    int MaxStack,
    ResourceCost Price,
    List<string> Tags
)
{
    internal static Item Decode(SoraReader reader)
    {
        return new Item(
            reader.ReadInt32(),
            reader.ReadString(),
            ItemTypeCodec.Decode(reader),
            reader.ReadInt32(),
            ResourceCost.Decode(reader),
            reader.ReadList(() => reader.ReadString())
        );
    }
}