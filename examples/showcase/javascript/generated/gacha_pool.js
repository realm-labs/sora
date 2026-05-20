
import { decodeResourceCost } from "./resource_cost.js";

export function decodeGachaPool(reader) {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        cost: decodeResourceCost(reader),
    };
}
