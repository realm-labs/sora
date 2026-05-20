
import { decodeElementType } from "./element_type.js";
import { decodeVec3 } from "./vec3.js";

export function decodeMonster(reader) {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        level: reader.readI32(),
        element: decodeElementType(reader),
        dropGroup: reader.readI32(),
        spawnPos: decodeVec3(reader),
    };
}
