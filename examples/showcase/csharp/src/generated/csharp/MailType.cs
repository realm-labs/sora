#nullable enable

namespace com.sora.showcase;

public enum MailType
{
    System,
    Event,
    Compensation,
}

internal static class MailTypeCodec
{
    internal static MailType Decode(SoraReader reader)
    {
        return reader.ReadUInt32() switch
        {
            0 => MailType.System,
            1 => MailType.Event,
            2 => MailType.Compensation,
            var value => throw new SoraReadException($"invalid enum ordinal {value} for MailType"),
        };
    }
}
