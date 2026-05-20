package com.sora.showcase;

public final class Stage {
    public final Integer id;
    public final String name;
    public final java.util.List<Integer> monsterIds;
    public final Integer recommendedPower;
    public final java.util.List<Reward> firstClearRewards;

    public Stage(
        Integer id,
        String name,
        java.util.List<Integer> monsterIds,
        Integer recommendedPower,
        java.util.List<Reward> firstClearRewards
    ) {
        this.id = id;
        this.name = name;
        this.monsterIds = monsterIds;
        this.recommendedPower = recommendedPower;
        this.firstClearRewards = firstClearRewards;
    }

    static Stage decode(SoraReader reader) {
        return new Stage(
            reader.readI32(),
            reader.readString(),
            reader.readList(() -> reader.readI32()),
            reader.readI32(),
            reader.readList(() -> Reward.decode(reader))
        );
    }
}
