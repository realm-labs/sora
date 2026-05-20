#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record CharacterSkill(
    int CharacterId,
    int SkillId,
    int UnlockLevel
)
{
    internal static CharacterSkill Decode(SoraReader reader)
    {
        return new CharacterSkill(
            reader.ReadInt32(),
            reader.ReadInt32(),
            reader.ReadInt32()
        );
    }
}