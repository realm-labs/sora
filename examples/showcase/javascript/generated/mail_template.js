
import { decodeMailType } from "./mail_type.js";
import { decodeReward } from "./reward.js";

export function decodeMailTemplate(reader) {
    return {
        id: reader.readI32(),
        mailType: decodeMailType(reader),
        titleKey: reader.readString(),
        bodyKey: reader.readString(),
        rewards: reader.readList(() => decodeReward(reader)),
    };
}
