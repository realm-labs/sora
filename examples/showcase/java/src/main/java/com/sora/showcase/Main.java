package com.sora.showcase;

import java.nio.file.Files;
import java.nio.file.Paths;

public final class Main {
    public static void main(String[] args) throws Exception {
        var bytes = Files.readAllBytes(Paths.get("..", "generated", "config.sora"));
        var config = SoraConfig.fromBytes(bytes);
        var sword = config.item().get(1001);
        var swordByName = config.item().getByName("Iron Sword");
        var quest = config.quest().get(5001);
        var settings = config.gameSettings().rows();

        check(sword.name.equals("Iron Sword"));
        check(swordByName.id == 1001);
        check(sword.itemType == ItemType.Weapon);
        check(quest.title.equals("First Trial"));
        check(quest.questType == QuestType.Main);
        check(quest.rewards.size() == 2);
        check(settings.startingGold == 100);
        check(config.stage().size() == 40);
        check(config.monster().size() == 80);
        check(config.localization().size() == 80);
        check(config.eventRule().size() == 20);

        var eventRule = config.eventRule().get(17001);
        check(eventRule.condition instanceof EventCondition.QuestCompleted);
        var condition = (EventCondition.QuestCompleted) eventRule.condition;
        check(condition.questId == 5002);
        check(eventRule.actions.get(0) instanceof RewardAction.AddItem);
        var firstAction = (RewardAction.AddItem) eventRule.actions.get(0);
        check(firstAction.itemId == 1007);

        System.out.println(
            "loaded " + config.item().size() + " items, " +
                config.skill().size() + " skills, " +
                config.quest().size() + " quests, " +
                config.stage().size() + " stages, " +
                config.eventRule().size() + " event rules; first quest rewards: " + quest.rewards.size()
        );
    }

    private static void check(boolean condition) {
        if (!condition) {
            throw new IllegalStateException("showcase assertion failed");
        }
    }
}
