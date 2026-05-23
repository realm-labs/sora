import 'dart:io';

import 'package:sora_showcase/sora_showcase.dart';

void main() {
  final bytes = File('../generated/client/config.json').readAsBytesSync();
  final config = SoraConfig.fromBytes(bytes);

  final sword = config.item.get(1001)!;
  final swordByName = config.item.getByName('Iron Sword');
  final quest = config.quest.get(5001)!;
  final settings = config.gameSettings.row;

  check(sword.name == 'Iron Sword');
  check(swordByName?.id == 1001);
  check(sword.itemType == ItemType.weapon);
  check(config.item.findByItemType(ItemType.weapon).any((item) => item.id == sword.id));
  check(quest.title == 'First Trial');
  check(quest.questType == QuestType.main);
  check(quest.rewards.length == 2);
  check(settings.startingGold == 100);
  check(config.stage.length == 40);
  check(config.monster.length == 80);
  check(config.localization.length == 80);
  check(config.eventRule.length == 20);

  final eventRule = config.eventRule.get(17001)!;
  check(eventRule.condition is EventConditionQuestCompleted);
  final condition = eventRule.condition as EventConditionQuestCompleted;
  check(condition.questId == 5002);
  check(eventRule.actions.first is RewardActionAddItem);
  final firstAction = eventRule.actions.first as RewardActionAddItem;
  check(firstAction.itemId == 1007);

  print(
    'loaded ${config.item.length} items, '
    '${config.skill.length} skills, '
    '${config.quest.length} quests, '
    '${config.stage.length} stages, '
    '${config.eventRule.length} event rules; '
    'first quest rewards: ${quest.rewards.length}',
  );
}

void check(bool condition) {
  if (!condition) {
    throw StateError('showcase assertion failed');
  }
}
