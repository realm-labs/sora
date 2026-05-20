import type { SoraReader } from "./sora_runtime.js";

export type Rarity =
    | "Common"
    | "Uncommon"
    | "Rare"
    | "Epic"
    | "Legendary";

export const Rarity = {
    Common: "Common",
    Uncommon: "Uncommon",
    Rare: "Rare",
    Epic: "Epic",
    Legendary: "Legendary",
} as const;
const values: Rarity[] = [
    Rarity.Common,
    Rarity.Uncommon,
    Rarity.Rare,
    Rarity.Epic,
    Rarity.Legendary,
];

export function decodeRarity(reader: SoraReader): Rarity {
    const ordinal = reader.readU32();
    const value = values[ordinal];
    if (value === undefined) {
        throw new Error(`invalid enum ordinal ${ordinal} for Rarity`);
    }
    return value;
}
