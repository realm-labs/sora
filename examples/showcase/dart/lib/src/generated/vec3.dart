import 'runtime.dart';

final class Vec3 {
  final double x;
  final double y;
  final double z;

  const Vec3({
    required this.x,
    required this.y,
    required this.z,
  });

  static Vec3 decode(SoraValue value) {
    final obj = value.asObject();
    return Vec3(
      x: obj.get("x").asDouble(),
      y: obj.get("y").asDouble(),
      z: obj.get("z").asDouble(),
    );
  }
}
