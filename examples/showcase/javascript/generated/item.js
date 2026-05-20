
import { decodeItemType } from "./item_type.js";
import { decodeResourceCost } from "./resource_cost.js";

export function decodeItem(reader) {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        itemType: decodeItemType(reader),
        maxStack: reader.readI32(),
        price: decodeResourceCost(reader),
        tags: reader.readList(() => reader.readString()),
    };
}
