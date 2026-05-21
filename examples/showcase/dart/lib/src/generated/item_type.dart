import 'runtime.dart';

enum ItemType {
  weapon,
  armor,
  currency,
  material,
  consumable;

  static ItemType decode(SoraValue value) {
    switch (value.asString()) {
      case 'Weapon':
        return ItemType.weapon;
      case 'Armor':
        return ItemType.armor;
      case 'Currency':
        return ItemType.currency;
      case 'Material':
        return ItemType.material;
      case 'Consumable':
        return ItemType.consumable;
      default:
        throw SoraReadException('invalid enum value `${value.asString()}` for ItemType');
    }
  }
}
