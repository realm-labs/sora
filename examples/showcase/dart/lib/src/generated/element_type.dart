import 'runtime.dart';

enum ElementType {
  fire,
  ice,
  lightning,
  physical;

  static ElementType decode(SoraValue value) {
    switch (value.asString()) {
      case 'Fire':
        return ElementType.fire;
      case 'Ice':
        return ElementType.ice;
      case 'Lightning':
        return ElementType.lightning;
      case 'Physical':
        return ElementType.physical;
      default:
        throw SoraReadException('invalid enum value `${value.asString()}` for ElementType');
    }
  }
}
