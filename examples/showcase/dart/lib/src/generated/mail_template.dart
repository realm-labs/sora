import 'runtime.dart';
import 'mail_type.dart';
import 'reward.dart';

final class MailTemplate {
  final int id;
  final MailType mailType;
  final String titleKey;
  final String bodyKey;
  final List<Reward> rewards;

  const MailTemplate({
    required this.id,
    required this.mailType,
    required this.titleKey,
    required this.bodyKey,
    required this.rewards,
  });

  static MailTemplate decode(SoraValue value) {
    final obj = value.asObject();
    return MailTemplate(
      id: obj.get("id").asInt(),
      mailType: MailType.decode(obj.get("mail_type")),
      titleKey: obj.get("title_key").asString(),
      bodyKey: obj.get("body_key").asString(),
      rewards: obj.get("rewards").asList((item) => Reward.decode(item)),
    );
  }
}
