import 'runtime.dart';

final class MailReward {
  final int mailId;
  final int seq;
  final int itemId;
  final int count;

  const MailReward({
    required this.mailId,
    required this.seq,
    required this.itemId,
    required this.count,
  });

  static MailReward decode(SoraValue value) {
    final obj = value.asObject();
    return MailReward(
      mailId: obj.get("mail_id").asInt(),
      seq: obj.get("seq").asInt(),
      itemId: obj.get("item_id").asInt(),
      count: obj.get("count").asInt(),
    );
  }
}
