#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record StatModifier(
    StatType Stat,
    float Value,
    bool IsPercent
)
{
    internal static StatModifier Decode(SoraReader reader)
    {
        return new StatModifier(
            StatTypeCodec.Decode(reader),
            reader.ReadFloat(),
            reader.ReadBool()
        );
    }
}