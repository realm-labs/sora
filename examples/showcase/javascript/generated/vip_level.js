
import { decodeResourceCost } from "./resource_cost.js";

export function decodeVipLevel(reader) {
    return {
        level: reader.readI32(),
        cost: decodeResourceCost(reader),
        perks: reader.readList(() => reader.readString()),
    };
}
