
import { decodeSkillEffect } from "./skill_effect.js";

export function decodeEquipmentSet(reader) {
    return {
        id: reader.readI32(),
        name: reader.readString(),
        itemIds: reader.readList(() => reader.readI32()),
        bonusEffect: decodeSkillEffect(reader),
    };
}
