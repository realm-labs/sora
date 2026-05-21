import 'runtime.dart';
import 'stat_modifier.dart';

final class Buff {
  final int id;
  final String name;
  final double duration;
  final List<StatModifier> modifiers;

  const Buff({
    required this.id,
    required this.name,
    required this.duration,
    required this.modifiers,
  });

  static Buff decode(SoraValue value) {
    final obj = value.asObject();
    return Buff(
      id: obj.get("id").asInt(),
      name: obj.get("name").asString(),
      duration: obj.get("duration").asDouble(),
      modifiers: obj.get("modifiers").asList((item) => StatModifier.decode(item)),
    );
  }
}
