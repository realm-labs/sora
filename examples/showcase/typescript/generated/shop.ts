import type { SoraReader } from "./sora_runtime.js";

import type { ResourceKind } from "./resource_kind.js";
import { decodeResourceKind } from "./resource_kind.js";


export interface Shop {
    id: number;
    name: string;
    currency: ResourceKind;
}

export function decodeShop(reader: SoraReader): Shop {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        currency: decodeResourceKind(reader),
    };
}
