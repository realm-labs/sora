export const StatType = {
    Hp: "Hp",
    Attack: "Attack",
    Defense: "Defense",
    Speed: "Speed",
    CritRate: "CritRate",
};
const values = [
    StatType.Hp,
    StatType.Attack,
    StatType.Defense,
    StatType.Speed,
    StatType.CritRate,
];

export function decodeStatType(reader) {
    const ordinal = reader.readU32();
    const value = values[ordinal];
    if (value === undefined) {
        throw new Error(`invalid enum ordinal ${ordinal} for StatType`);
    }
    return value;
}
