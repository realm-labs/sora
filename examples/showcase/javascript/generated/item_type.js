export const ItemType = {
    Weapon: "Weapon",
    Armor: "Armor",
    Currency: "Currency",
    Material: "Material",
    Consumable: "Consumable",
};
const values = [
    ItemType.Weapon,
    ItemType.Armor,
    ItemType.Currency,
    ItemType.Material,
    ItemType.Consumable,
];

export function decodeItemType(reader) {
    const ordinal = reader.readU32();
    const value = values[ordinal];
    if (value === undefined) {
        throw new Error(`invalid enum ordinal ${ordinal} for ItemType`);
    }
    return value;
}
