import type { SoraReader } from "./sora_runtime.js";

export type ItemType =
    | "Weapon"
    | "Armor"
    | "Currency"
    | "Material"
    | "Consumable";

export const ItemType = {
    Weapon: "Weapon",
    Armor: "Armor",
    Currency: "Currency",
    Material: "Material",
    Consumable: "Consumable",
} as const;
const values: ItemType[] = [
    ItemType.Weapon,
    ItemType.Armor,
    ItemType.Currency,
    ItemType.Material,
    ItemType.Consumable,
];

export function decodeItemType(reader: SoraReader): ItemType {
    const ordinal = reader.readU32();
    const value = values[ordinal];
    if (value === undefined) {
        throw new Error(`invalid enum ordinal ${ordinal} for ItemType`);
    }
    return value;
}
