import 'runtime.dart';

sealed class EventCondition {
  const EventCondition();

  static EventCondition decode(SoraValue value) {
    final obj = value.asObject();
    final tag = obj.get('type').asString();
    switch (tag) {
      case 'LevelAtLeast':
        return EventConditionLevelAtLeast(
          level: obj.get("level").asInt(),
        );
      case 'QuestCompleted':
        return EventConditionQuestCompleted(
          questId: obj.get("quest_id").asInt(),
        );
      case 'HasItem':
        return EventConditionHasItem(
          itemId: obj.get("item_id").asInt(),
          count: obj.get("count").asInt(),
        );
      default:
        throw SoraReadException('invalid union tag `$tag` for EventCondition');
    }
  }
}

final class EventConditionLevelAtLeast extends EventCondition {
  final int level;

  const EventConditionLevelAtLeast({
    required this.level,
  });
}

final class EventConditionQuestCompleted extends EventCondition {
  final int questId;

  const EventConditionQuestCompleted({
    required this.questId,
  });
}

final class EventConditionHasItem extends EventCondition {
  final int itemId;
  final int count;

  const EventConditionHasItem({
    required this.itemId,
    required this.count,
  });
}
