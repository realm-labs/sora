import 'runtime.dart';

final class CharacterSkill {
  final int characterId;
  final int skillId;
  final int unlockLevel;

  const CharacterSkill({
    required this.characterId,
    required this.skillId,
    required this.unlockLevel,
  });

  static CharacterSkill decode(SoraValue value) {
    final obj = value.asObject();
    return CharacterSkill(
      characterId: obj.get("character_id").asInt(),
      skillId: obj.get("skill_id").asInt(),
      unlockLevel: obj.get("unlock_level").asInt(),
    );
  }
}
