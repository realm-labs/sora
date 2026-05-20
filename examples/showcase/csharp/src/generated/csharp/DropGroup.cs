#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record DropGroup(
    int Id,
    string Name
)
{
    internal static DropGroup Decode(SoraReader reader)
    {
        return new DropGroup(
            reader.ReadInt32(),
            reader.ReadString()
        );
    }
}