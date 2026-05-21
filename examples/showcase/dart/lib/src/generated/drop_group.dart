import 'runtime.dart';

final class DropGroup {
  final int id;
  final String name;

  const DropGroup({
    required this.id,
    required this.name,
  });

  static DropGroup decode(SoraValue value) {
    final obj = value.asObject();
    return DropGroup(
      id: obj.get("id").asInt(),
      name: obj.get("name").asString(),
    );
  }
}
