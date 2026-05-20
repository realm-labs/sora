
import { decodeElementType } from "./element_type.js";

export function decodeSkillEffect(reader) {
    return {
        element: decodeElementType(reader),
        power: reader.readI32(),
        radius: reader.readF32(),
    };
}
