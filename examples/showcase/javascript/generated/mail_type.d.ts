import type { SoraReader } from "./sora_runtime.js";

export type MailType =
    | "System"
    | "Event"
    | "Compensation";

export declare const MailType: {
    readonly System: "System";
    readonly Event: "Event";
    readonly Compensation: "Compensation";
};

export declare function decodeMailType(reader: SoraReader): MailType;
