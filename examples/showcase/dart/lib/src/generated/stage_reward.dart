import 'runtime.dart';

final class StageReward {
  final int stageId;
  final int seq;
  final int itemId;
  final int count;

  const StageReward({
    required this.stageId,
    required this.seq,
    required this.itemId,
    required this.count,
  });

  static StageReward decode(SoraValue value) {
    final obj = value.asObject();
    return StageReward(
      stageId: obj.get("stage_id").asInt(),
      seq: obj.get("seq").asInt(),
      itemId: obj.get("item_id").asInt(),
      count: obj.get("count").asInt(),
    );
  }
}
