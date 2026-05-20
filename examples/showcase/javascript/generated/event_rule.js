
import { decodeEventCondition } from "./event_condition.js";
import { decodeRewardAction } from "./reward_action.js";

export function decodeEventRule(reader) {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        condition: decodeEventCondition(reader),
        actions: reader.readList(() => decodeRewardAction(reader)),
    };
}
