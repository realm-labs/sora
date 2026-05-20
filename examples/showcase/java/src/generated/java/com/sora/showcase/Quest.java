package com.sora.showcase;

public final class Quest {
    public final Integer id;
    public final QuestType questType;
    public final String title;
    public final Integer requiredItem;
    public final java.util.List<Integer> unlockSkills;
    public final Vec3 startPos;
    public final java.util.List<Reward> rewards;

    public Quest(
        Integer id,
        QuestType questType,
        String title,
        Integer requiredItem,
        java.util.List<Integer> unlockSkills,
        Vec3 startPos,
        java.util.List<Reward> rewards
    ) {
        this.id = id;
        this.questType = questType;
        this.title = title;
        this.requiredItem = requiredItem;
        this.unlockSkills = unlockSkills;
        this.startPos = startPos;
        this.rewards = rewards;
    }

    static Quest decode(SoraReader reader) {
        return new Quest(
            reader.readI32(),
            QuestType.decode(reader),
            reader.readString(),
            reader.readI32(),
            reader.readList(() -> reader.readI32()),
            Vec3.decode(reader),
            reader.readList(() -> Reward.decode(reader))
        );
    }
}