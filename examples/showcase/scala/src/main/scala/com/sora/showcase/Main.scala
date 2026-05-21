package com.sora.showcase

import java.nio.file.Files
import java.nio.file.Paths

object Main {
  def main(args: Array[String]): Unit = {
    val bytes = Files.readAllBytes(Paths.get("..", "generated", "config.sora"))
    val config = SoraConfig.fromBytes(bytes)
    val sword = config.item.get(1001).getOrElse(sys.error("item 1001"))
    val swordByName = config.item.getByName("Iron Sword").getOrElse(sys.error("Iron Sword"))
    val quest = config.quest.get(5001).getOrElse(sys.error("quest 5001"))
    val settings = config.gameSettings.value

    check(sword.name == "Iron Sword")
    check(swordByName.id == 1001)
    check(sword.itemType == ItemType.Weapon)
    check(config.item.findByItemType(ItemType.Weapon).exists(_.id == sword.id))
    check(quest.title == "First Trial")
    check(quest.questType == QuestType.Main)
    check(quest.rewards.size == 2)
    check(settings.startingGold == 100)
    check(config.stage.size == 40)
    check(config.monster.size == 80)
    check(config.localization.size == 80)
    check(config.eventRule.size == 20)

    val eventRule = config.eventRule.get(17001).getOrElse(sys.error("event rule 17001"))
    val conditionOk = eventRule.condition match {
      case EventCondition.QuestCompleted(questId) => questId == 5002
      case _ => false
    }
    check(conditionOk)

    val actionOk = eventRule.actions.headOption.exists {
      case RewardAction.AddItem(itemId, _) => itemId == 1007
      case _ => false
    }
    check(actionOk)

    println(
      s"loaded ${config.item.size} items, " +
        s"${config.skill.size} skills, " +
        s"${config.quest.size} quests, " +
        s"${config.stage.size} stages, " +
        s"${config.eventRule.size} event rules; first quest rewards: ${quest.rewards.size}"
    )
  }

  private def check(condition: Boolean): Unit = {
    if (!condition) {
      throw new IllegalStateException("showcase assertion failed")
    }
  }
}
