import type { SoraReader } from "./sora_runtime.js";

export type StatType =
    | "Hp"
    | "Attack"
    | "Defense"
    | "Speed"
    | "CritRate";

export const StatType = {
    Hp: "Hp",
    Attack: "Attack",
    Defense: "Defense",
    Speed: "Speed",
    CritRate: "CritRate",
} as const;
const values: StatType[] = [
    StatType.Hp,
    StatType.Attack,
    StatType.Defense,
    StatType.Speed,
    StatType.CritRate,
];

export function decodeStatType(reader: SoraReader): StatType {
    const ordinal = reader.readU32();
    const value = values[ordinal];
    if (value === undefined) {
        throw new Error(`invalid enum ordinal ${ordinal} for StatType`);
    }
    return value;
}
