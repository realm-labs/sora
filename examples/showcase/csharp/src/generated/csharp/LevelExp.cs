#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record LevelExp(
    int Level,
    long Exp,
    string? UnlockFeature
)
{
    internal static LevelExp Decode(SoraReader reader)
    {
        return new LevelExp(
            reader.ReadInt32(),
            reader.ReadInt64(),
            reader.ReadOptional(() => reader.ReadString())
        );
    }
}
