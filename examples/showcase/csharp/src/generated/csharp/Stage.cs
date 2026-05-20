#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record Stage(
    int Id,
    string Name,
    List<int> MonsterIds,
    int RecommendedPower,
    List<Reward> FirstClearRewards
)
{
    internal static Stage Decode(SoraReader reader)
    {
        return new Stage(
            reader.ReadInt32(),
            reader.ReadString(),
            reader.ReadList(() => reader.ReadInt32()),
            reader.ReadInt32(),
            reader.ReadList(() => Reward.Decode(reader))
        );
    }
}
