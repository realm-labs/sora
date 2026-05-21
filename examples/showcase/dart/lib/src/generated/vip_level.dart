import 'runtime.dart';
import 'resource_cost.dart';

final class VipLevel {
  final int level;
  final ResourceCost cost;
  final List<String> perks;

  const VipLevel({
    required this.level,
    required this.cost,
    required this.perks,
  });

  static VipLevel decode(SoraValue value) {
    final obj = value.asObject();
    return VipLevel(
      level: obj.get("level").asInt(),
      cost: ResourceCost.decode(obj.get("cost")),
      perks: obj.get("perks").asList((item) => item.asString()),
    );
  }
}
