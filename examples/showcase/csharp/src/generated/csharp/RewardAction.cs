#nullable enable

using System.Collections.Generic;

namespace com.sora.showcase;

public abstract record RewardAction
{
    public sealed record AddItem(
        int ItemId,
        int Count
    ) : RewardAction;
    public sealed record AddBuff(
        int BuffId,
        float Duration
    ) : RewardAction;
    public sealed record UnlockStage(
        int StageId
    ) : RewardAction;
    public sealed record SendMail(
        int MailId
    ) : RewardAction;
    internal static RewardAction Decode(SoraReader reader)
    {
        return reader.ReadUInt32() switch
        {
            0 => new AddItem(
                reader.ReadInt32(),
                reader.ReadInt32()
            ),
            1 => new AddBuff(
                reader.ReadInt32(),
                reader.ReadFloat()
            ),
            2 => new UnlockStage(
                reader.ReadInt32()
            ),
            3 => new SendMail(
                reader.ReadInt32()
            ),
            var value => throw new SoraReadException($"invalid union ordinal {value} for RewardAction"),
        };
    }
}
