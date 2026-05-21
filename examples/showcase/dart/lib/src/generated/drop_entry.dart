import 'runtime.dart';

final class DropEntry {
  final int groupId;
  final int seq;
  final int itemId;
  final int count;
  final double weight;

  const DropEntry({
    required this.groupId,
    required this.seq,
    required this.itemId,
    required this.count,
    required this.weight,
  });

  static DropEntry decode(SoraValue value) {
    final obj = value.asObject();
    return DropEntry(
      groupId: obj.get("group_id").asInt(),
      seq: obj.get("seq").asInt(),
      itemId: obj.get("item_id").asInt(),
      count: obj.get("count").asInt(),
      weight: obj.get("weight").asDouble(),
    );
  }
}
