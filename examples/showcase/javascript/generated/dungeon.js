
import { decodeResourceCost } from "./resource_cost.js";

export function decodeDungeon(reader) {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        stageIds: reader.readList(() => reader.readI32()),
        entryCost: decodeResourceCost(reader),
    };
}
