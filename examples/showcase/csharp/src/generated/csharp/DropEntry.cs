#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record DropEntry(
    int GroupId,
    int Seq,
    int ItemId,
    int Count,
    float Weight
)
{
    internal static DropEntry Decode(SoraReader reader)
    {
        return new DropEntry(
            reader.ReadInt32(),
            reader.ReadInt32(),
            reader.ReadInt32(),
            reader.ReadInt32(),
            reader.ReadFloat()
        );
    }
}