import type { SoraReader } from "./sora_runtime.js";

export type MailType =
    | "System"
    | "Event"
    | "Compensation";

export const MailType = {
    System: "System",
    Event: "Event",
    Compensation: "Compensation",
} as const;
const values: MailType[] = [
    MailType.System,
    MailType.Event,
    MailType.Compensation,
];

export function decodeMailType(reader: SoraReader): MailType {
    const ordinal = reader.readU32();
    const value = values[ordinal];
    if (value === undefined) {
        throw new Error(`invalid enum ordinal ${ordinal} for MailType`);
    }
    return value;
}
