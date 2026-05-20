package game_config_showcase.smoke

import game_config_showcase.ItemType
import game_config_showcase.QuestType
import game_config_showcase.SoraConfig
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

    println(
        "loaded ${config.itemValues().size} items, " +
            "${config.skillValues().size} skills, " +
            "${config.questValues().size} quests; first quest rewards: ${quest.rewards.size}",
    )
}
