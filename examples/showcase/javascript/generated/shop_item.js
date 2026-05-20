
import { decodeResourceCost } from "./resource_cost.js";

export function decodeShopItem(reader) {
    return {
        shopId: reader.readI32(),
        seq: reader.readI32(),
        itemId: reader.readI32(),
        price: decodeResourceCost(reader),
        dailyLimit: reader.readOptional(() => reader.readI32()),
    };
}
