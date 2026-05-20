import type { SoraReader } from "./sora_runtime.js";

import type { ResourceKind } from "./resource_kind.js";


export interface ResourceCost {
    kind: ResourceKind;
    id: number;
    count: number;
}

export declare function decodeResourceCost(reader: SoraReader): ResourceCost;
