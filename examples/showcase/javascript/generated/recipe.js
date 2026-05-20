
import { decodeResourceCost } from "./resource_cost.js";

export function decodeRecipe(reader) {
    return {
        id: reader.readI32(),
        resultItem: reader.readI32(),
        materials: reader.readList(() => decodeResourceCost(reader)),
    };
}
