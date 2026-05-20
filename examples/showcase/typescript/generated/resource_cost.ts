import type { SoraReader } from "./sora_runtime.js";

import type { ResourceKind } from "./resource_kind.js";
import { decodeResourceKind } from "./resource_kind.js";


export interface ResourceCost {
    kind: ResourceKind;
    id: number;
    count: number;
}

export function decodeResourceCost(reader: SoraReader): ResourceCost {
    return {
        kind: decodeResourceKind(reader),
        id: reader.readI32(),
        count: reader.readI32(),
    };
}
