
import { decodeReward } from "./reward.js";

export function decodeStage(reader) {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        monsterIds: reader.readList(() => reader.readI32()),
        recommendedPower: reader.readI32(),
        firstClearRewards: reader.readList(() => decodeReward(reader)),
    };
}
