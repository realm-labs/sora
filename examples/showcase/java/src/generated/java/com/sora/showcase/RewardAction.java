package com.sora.showcase;

public interface RewardAction {
    final class AddItem implements RewardAction {
        public final Integer itemId;
        public final Integer count;

        public AddItem(
            Integer itemId,
            Integer count
        ) {
            this.itemId = itemId;
            this.count = count;
        }
    }
    final class AddBuff implements RewardAction {
        public final Integer buffId;
        public final Float duration;

        public AddBuff(
            Integer buffId,
            Float duration
        ) {
            this.buffId = buffId;
            this.duration = duration;
        }
    }
    final class UnlockStage implements RewardAction {
        public final Integer stageId;

        public UnlockStage(
            Integer stageId
        ) {
            this.stageId = stageId;
        }
    }
    final class SendMail implements RewardAction {
        public final Integer mailId;

        public SendMail(
            Integer mailId
        ) {
            this.mailId = mailId;
        }
    }
    static RewardAction decode(SoraReader reader) {
        switch (reader.readU32()) {
            case 0:
                return new AddItem(
                    reader.readI32(),
                    reader.readI32()
                );
            case 1:
                return new AddBuff(
                    reader.readI32(),
                    reader.readF32()
                );
            case 2:
                return new UnlockStage(
                    reader.readI32()
                );
            case 3:
                return new SendMail(
                    reader.readI32()
                );
            default:
                throw new SoraReadException("invalid union ordinal for RewardAction");
        }
    }
}