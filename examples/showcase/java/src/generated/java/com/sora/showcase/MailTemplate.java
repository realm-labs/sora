package com.sora.showcase;

public final class MailTemplate {
    public final Integer id;
    public final MailType mailType;
    public final String titleKey;
    public final String bodyKey;
    public final java.util.List<Reward> rewards;

    public MailTemplate(
        Integer id,
        MailType mailType,
        String titleKey,
        String bodyKey,
        java.util.List<Reward> rewards
    ) {
        this.id = id;
        this.mailType = mailType;
        this.titleKey = titleKey;
        this.bodyKey = bodyKey;
        this.rewards = rewards;
    }

    static MailTemplate decode(SoraReader reader) {
        return new MailTemplate(
            reader.readI32(),
            MailType.decode(reader),
            reader.readString(),
            reader.readString(),
            reader.readList(() -> Reward.decode(reader))
        );
    }
}
