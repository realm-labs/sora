import 'runtime.dart';
import 'element_type.dart';
import 'vec3.dart';

final class Monster {
  final int id;
  final String name;
  final int level;
  final ElementType element;
  final int dropGroup;
  final Vec3 spawnPos;

  const Monster({
    required this.id,
    required this.name,
    required this.level,
    required this.element,
    required this.dropGroup,
    required this.spawnPos,
  });

  static Monster decode(SoraValue value) {
    final obj = value.asObject();
    return Monster(
      id: obj.get("id").asInt(),
      name: obj.get("name").asString(),
      level: obj.get("level").asInt(),
      element: ElementType.decode(obj.get("element")),
      dropGroup: obj.get("drop_group").asInt(),
      spawnPos: Vec3.decode(obj.get("spawn_pos")),
    );
  }
}
