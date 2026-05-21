import 'runtime.dart';

enum Rarity {
  common,
  uncommon,
  rare,
  epic,
  legendary;

  static Rarity decode(SoraValue value) {
    switch (value.asString()) {
      case 'Common':
        return Rarity.common;
      case 'Uncommon':
        return Rarity.uncommon;
      case 'Rare':
        return Rarity.rare;
      case 'Epic':
        return Rarity.epic;
      case 'Legendary':
        return Rarity.legendary;
      default:
        throw SoraReadException('invalid enum value `${value.asString()}` for Rarity');
    }
  }
}
