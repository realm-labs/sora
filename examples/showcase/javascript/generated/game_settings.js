
import { decodeVec3 } from "./vec3.js";

export function decodeGameSettings(reader) {
    return {
        version: reader.readString(),
        dailyResetHour: reader.readI32(),
        startingGold: reader.readI32(),
        spawnPos: decodeVec3(reader),
        starterItems: reader.readList(() => reader.readI32()),
    };
}
