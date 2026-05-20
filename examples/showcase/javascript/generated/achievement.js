
import { decodeResourceCost } from "./resource_cost.js";

export function decodeAchievement(reader) {
    return {
        id: reader.readI32(),
        titleKey: reader.readString(),
        targetCount: reader.readI64(),
        reward: decodeResourceCost(reader),
    };
}
