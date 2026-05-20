#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record Localization(
    string Key,
    string ZhCn,
    string EnUs,
    string? Note
)
{
    internal static Localization Decode(SoraReader reader)
    {
        return new Localization(
            reader.ReadString(),
            reader.ReadString(),
            reader.ReadString(),
            reader.ReadOptional(() => reader.ReadString())
        );
    }
}
