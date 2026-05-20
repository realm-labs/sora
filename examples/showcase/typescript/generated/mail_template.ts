import type { SoraReader } from "./sora_runtime.js";

import type { MailType } from "./mail_type.js";
import { decodeMailType } from "./mail_type.js";

import type { Reward } from "./reward.js";
import { decodeReward } from "./reward.js";


export interface MailTemplate {
    id: number;
    mailType: MailType;
    titleKey: string;
    bodyKey: string;
    rewards: Reward[];
}

export function decodeMailTemplate(reader: SoraReader): MailTemplate {
    return {
        id: reader.readI32(),
        mailType: decodeMailType(reader),
        titleKey: reader.readString(),
        bodyKey: reader.readString(),
        rewards: reader.readList(() => decodeReward(reader)),
    };
}
