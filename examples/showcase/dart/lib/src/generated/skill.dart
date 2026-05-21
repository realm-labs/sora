import 'runtime.dart';
import 'element_type.dart';
import 'resource_cost.dart';
import 'skill_effect.dart';
import 'vec3.dart';

final class Skill {
  final int id;
  final String name;
  final ElementType element;
  final ResourceCost cost;
  final SkillEffect effect;
  final int requiredLevel;
  final int? requiredItem;
  final Vec3 castOrigin;

  const Skill({
    required this.id,
    required this.name,
    required this.element,
    required this.cost,
    required this.effect,
    required this.requiredLevel,
    required this.requiredItem,
    required this.castOrigin,
  });

  static Skill decode(SoraValue value) {
    final obj = value.asObject();
    return Skill(
      id: obj.get("id").asInt(),
      name: obj.get("name").asString(),
      element: ElementType.decode(obj.get("element")),
      cost: ResourceCost.decode(obj.get("cost")),
      effect: SkillEffect.decode(obj.get("effect")),
      requiredLevel: obj.get("required_level").asInt(),
      requiredItem: obj.get("required_item").isNull ? null : obj.get("required_item").asInt(),
      castOrigin: Vec3.decode(obj.get("cast_origin")),
    );
  }
}
