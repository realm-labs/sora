package com.sora.showcase

import java.nio.file.Paths

fun main() {
    val bytes = Paths.get("..", "generated", "config.sora").toFile().readBytes()
    val config = SoraConfig.fromBytes(bytes)
    val sword = config.item[1001] ?: error("item 1001")
    val swordByName = config.item.getByName("Iron Sword") ?: error("Iron Sword")
    val quest = config.quest[5001] ?: error("quest 5001")
    val settings = config.gameSettings.value()

    check(sword.name == "Iron Sword")
    check(swordByName.id == 1001)
    check(sword.itemType == ItemType.Weapon)
    check(config.item.findByItemType(ItemType.Weapon).any { it.id == sword.id })
    check(quest.title == "First Trial")
    check(quest.questType == QuestType.Main)
    check(quest.rewards.size == 2)
    check(settings.startingGold == 100)
    check(config.stage.values().size == 40)
    check(config.monster.values().size == 80)
    check(config.localization.values().size == 80)
    check(config.eventRule.values().size == 20)

    val eventRule = config.eventRule[17001] ?: error("event rule 17001")
    val condition = eventRule.condition
    check(condition is EventCondition.QuestCompleted && condition.questId == 5002)
    val firstAction = eventRule.actions.first()
    check(firstAction is RewardAction.AddItem && firstAction.itemId == 1007)

    println(
        "loaded ${config.item.values().size} items, " +
            "${config.skill.values().size} skills, " +
            "${config.quest.values().size} quests, " +
            "${config.stage.values().size} stages, " +
            "${config.eventRule.values().size} event rules; first quest rewards: ${quest.rewards.size}",
    )
}
