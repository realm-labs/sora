import 'runtime.dart';

final class QuestReward {
  final int questId;
  final int seq;
  final int itemId;
  final int count;

  const QuestReward({
    required this.questId,
    required this.seq,
    required this.itemId,
    required this.count,
  });

  static QuestReward decode(SoraValue value) {
    final obj = value.asObject();
    return QuestReward(
      questId: obj.get("quest_id").asInt(),
      seq: obj.get("seq").asInt(),
      itemId: obj.get("item_id").asInt(),
      count: obj.get("count").asInt(),
    );
  }
}
