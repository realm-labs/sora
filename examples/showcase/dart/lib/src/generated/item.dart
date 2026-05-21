import 'runtime.dart';
import 'item_type.dart';
import 'resource_cost.dart';

final class Item {
  final int id;
  final String name;
  final ItemType itemType;
  final int maxStack;
  final ResourceCost price;
  final List<String> tags;

  const Item({
    required this.id,
    required this.name,
    required this.itemType,
    required this.maxStack,
    required this.price,
    required this.tags,
  });

  static Item decode(SoraValue value) {
    final obj = value.asObject();
    return Item(
      id: obj.get("id").asInt(),
      name: obj.get("name").asString(),
      itemType: ItemType.decode(obj.get("item_type")),
      maxStack: obj.get("max_stack").asInt(),
      price: ResourceCost.decode(obj.get("price")),
      tags: obj.get("tags").asList((item) => item.asString()),
    );
  }
}
