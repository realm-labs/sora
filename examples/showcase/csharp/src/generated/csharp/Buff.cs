#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record Buff(
    int Id,
    string Name,
    float Duration,
    List<StatModifier> Modifiers
)
{
    internal static Buff Decode(SoraReader reader)
    {
        return new Buff(
            reader.ReadInt32(),
            reader.ReadString(),
            reader.ReadFloat(),
            reader.ReadList(() => StatModifier.Decode(reader))
        );
    }
}