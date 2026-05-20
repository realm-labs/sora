import type { SoraReader } from "./sora_runtime.js";

export type QuestType =
    | "Main"
    | "Side"
    | "Daily";

export declare const QuestType: {
    readonly Main: "Main";
    readonly Side: "Side";
    readonly Daily: "Daily";
};

export declare function decodeQuestType(reader: SoraReader): QuestType;
