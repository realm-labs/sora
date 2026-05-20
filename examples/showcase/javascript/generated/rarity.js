export const Rarity = {
    Common: "Common",
    Uncommon: "Uncommon",
    Rare: "Rare",
    Epic: "Epic",
    Legendary: "Legendary",
};
const values = [
    Rarity.Common,
    Rarity.Uncommon,
    Rarity.Rare,
    Rarity.Epic,
    Rarity.Legendary,
];

export function decodeRarity(reader) {
    const ordinal = reader.readU32();
    const value = values[ordinal];
    if (value === undefined) {
        throw new Error(`invalid enum ordinal ${ordinal} for Rarity`);
    }
    return value;
}
