
import { decodeResourceKind } from "./resource_kind.js";

export function decodeShop(reader) {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        currency: decodeResourceKind(reader),
    };
}
