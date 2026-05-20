
import { decodeQuestType } from "./quest_type.js";
import { decodeReward } from "./reward.js";
import { decodeVec3 } from "./vec3.js";

export function decodeQuest(reader) {
    return {
        id: reader.readI32(),
        questType: decodeQuestType(reader),
        title: reader.readString(),
        requiredItem: reader.readI32(),
        unlockSkills: reader.readList(() => reader.readI32()),
        startPos: decodeVec3(reader),
        rewards: reader.readList(() => decodeReward(reader)),
    };
}
