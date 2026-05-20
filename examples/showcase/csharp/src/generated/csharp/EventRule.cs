#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record EventRule(
    int Id,
    string Name,
    EventCondition Condition,
    List<RewardAction> Actions
)
{
    internal static EventRule Decode(SoraReader reader)
    {
        return new EventRule(
            reader.ReadInt32(),
            reader.ReadString(),
            EventCondition.Decode(reader),
            reader.ReadList(() => RewardAction.Decode(reader))
        );
    }
}
