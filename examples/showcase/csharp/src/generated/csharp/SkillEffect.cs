#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record SkillEffect(
    ElementType Element,
    int Power,
    float Radius
)
{
    internal static SkillEffect Decode(SoraReader reader)
    {
        return new SkillEffect(
            ElementTypeCodec.Decode(reader),
            reader.ReadInt32(),
            reader.ReadFloat()
        );
    }
}
