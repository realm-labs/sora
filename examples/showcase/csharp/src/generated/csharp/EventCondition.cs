#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public abstract record EventCondition
{
    public sealed record LevelAtLeast(
        int Level
    ) : EventCondition;
    public sealed record QuestCompleted(
        int QuestId
    ) : EventCondition;
    public sealed record HasItem(
        int ItemId,
        int Count
    ) : EventCondition;
    internal static EventCondition Decode(SoraReader reader)
    {
        return reader.ReadUInt32() switch
        {
            0 => new LevelAtLeast(
                reader.ReadInt32()
            ),
            1 => new QuestCompleted(
                reader.ReadInt32()
            ),
            2 => new HasItem(
                reader.ReadInt32(),
                reader.ReadInt32()
            ),
            var value => throw new SoraReadException($"invalid union ordinal {value} for EventCondition"),
        };
    }
}
