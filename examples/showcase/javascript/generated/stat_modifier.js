
import { decodeStatType } from "./stat_type.js";

export function decodeStatModifier(reader) {
    return {
        stat: decodeStatType(reader),
        value: reader.readF32(),
        isPercent: reader.readBool(),
    };
}
