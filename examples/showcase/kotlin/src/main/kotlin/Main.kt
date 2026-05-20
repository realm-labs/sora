package com.sora.showcase

import java.nio.file.Paths

fun main() {
    val bytes = Paths.get("..", "generated", "config.sora").toFile().readBytes()
    val config = SoraConfig.fromBytes(bytes)
    val sword = config.getItem(1001) ?: error("item 1001")
    val quest = config.getQuest(5001) ?: error("quest 5001")
    val settings = config.gameSettingsRow()

    check(sword.name == "Iron Sword")
    check(sword.itemType == ItemType.Weapon)
    check(quest.title == "First Trial")
    check(quest.questType == QuestType.Main)
    check(quest.rewards.size == 2)
    check(settings.startingGold == 100)
    check(config.stageValues().size == 40)
    check(config.monsterValues().size == 80)
    check(config.localizationValues().size == 80)
    check(config.eventRuleValues().size == 20)

    val eventRule = config.getEventRule(17001) ?: error("event rule 17001")
    val condition = eventRule.condition
    check(condition is EventCondition.QuestCompleted && condition.questId == 5002)
    val firstAction = eventRule.actions.first()
    check(firstAction is RewardAction.AddItem && firstAction.itemId == 1007)

    println(
        "loaded ${config.itemValues().size} items, " +
            "${config.skillValues().size} skills, " +
            "${config.questValues().size} quests, " +
            "${config.stageValues().size} stages, " +
            "${config.eventRuleValues().size} event rules; first quest rewards: ${quest.rewards.size}",
    )
}
