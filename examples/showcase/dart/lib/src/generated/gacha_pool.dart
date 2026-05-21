import 'runtime.dart';
import 'resource_cost.dart';

final class GachaPool {
  final int id;
  final String name;
  final ResourceCost cost;

  const GachaPool({
    required this.id,
    required this.name,
    required this.cost,
  });

  static GachaPool decode(SoraValue value) {
    final obj = value.asObject();
    return GachaPool(
      id: obj.get("id").asInt(),
      name: obj.get("name").asString(),
      cost: ResourceCost.decode(obj.get("cost")),
    );
  }
}
