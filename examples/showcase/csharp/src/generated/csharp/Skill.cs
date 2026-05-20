#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record Skill(
    int Id,
    string Name,
    ElementType Element,
    ResourceCost Cost,
    SkillEffect Effect,
    int RequiredLevel,
    int? RequiredItem,
    Vec3 CastOrigin
)
{
    internal static Skill Decode(SoraReader reader)
    {
        return new Skill(
            reader.ReadInt32(),
            reader.ReadString(),
            ElementTypeCodec.Decode(reader),
            ResourceCost.Decode(reader),
            SkillEffect.Decode(reader),
            reader.ReadInt32(),
            reader.ReadOptional(() => reader.ReadInt32()),
            Vec3.Decode(reader)
        );
    }
}
