package com.sora.showcase;

public final class EventRule {
    public final Integer id;
    public final String name;
    public final EventCondition condition;
    public final java.util.List<RewardAction> actions;

    public EventRule(
        Integer id,
        String name,
        EventCondition condition,
        java.util.List<RewardAction> actions
    ) {
        this.id = id;
        this.name = name;
        this.condition = condition;
        this.actions = actions;
    }

    static EventRule decode(SoraReader reader) {
        return new EventRule(
            reader.readI32(),
            reader.readString(),
            EventCondition.decode(reader),
            reader.readList(() -> RewardAction.decode(reader))
        );
    }
}