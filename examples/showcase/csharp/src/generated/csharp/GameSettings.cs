#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public sealed record GameSettings(
    string Version,
    int DailyResetHour,
    int StartingGold,
    Vec3 SpawnPos,
    List<int> StarterItems
)
{
    internal static GameSettings Decode(SoraReader reader)
    {
        return new GameSettings(
            reader.ReadString(),
            reader.ReadInt32(),
            reader.ReadInt32(),
            Vec3.Decode(reader),
            reader.ReadList(() => reader.ReadInt32())
        );
    }
}
