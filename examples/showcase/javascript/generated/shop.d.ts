import type { SoraReader } from "./sora_runtime.js";

import type { ResourceKind } from "./resource_kind.js";


export interface Shop {
    id: number;
    name: string;
    currency: ResourceKind;
}

export declare function decodeShop(reader: SoraReader): Shop;
