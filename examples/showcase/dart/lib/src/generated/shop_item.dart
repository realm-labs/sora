import 'runtime.dart';
import 'resource_cost.dart';

final class ShopItem {
  final int shopId;
  final int seq;
  final int itemId;
  final ResourceCost price;
  final int? dailyLimit;

  const ShopItem({
    required this.shopId,
    required this.seq,
    required this.itemId,
    required this.price,
    required this.dailyLimit,
  });

  static ShopItem decode(SoraValue value) {
    final obj = value.asObject();
    return ShopItem(
      shopId: obj.get("shop_id").asInt(),
      seq: obj.get("seq").asInt(),
      itemId: obj.get("item_id").asInt(),
      price: ResourceCost.decode(obj.get("price")),
      dailyLimit: obj.get("daily_limit").isNull ? null : obj.get("daily_limit").asInt(),
    );
  }
}
