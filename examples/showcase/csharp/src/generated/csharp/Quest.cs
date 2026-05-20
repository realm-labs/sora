#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record Quest(
    int Id,
    QuestType QuestType,
    string Title,
    int RequiredItem,
    List<int> UnlockSkills,
    Vec3 StartPos,
    List<Reward> Rewards
)
{
    internal static Quest Decode(SoraReader reader)
    {
        return new Quest(
            reader.ReadInt32(),
            QuestTypeCodec.Decode(reader),
            reader.ReadString(),
            reader.ReadInt32(),
            reader.ReadList(() => reader.ReadInt32()),
            Vec3.Decode(reader),
            reader.ReadList(() => Reward.Decode(reader))
        );
    }
}