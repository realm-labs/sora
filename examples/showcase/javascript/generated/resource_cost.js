
import { decodeResourceKind } from "./resource_kind.js";

export function decodeResourceCost(reader) {
    return {
        kind: decodeResourceKind(reader),
        id: reader.readI32(),
        count: reader.readI32(),
    };
}
