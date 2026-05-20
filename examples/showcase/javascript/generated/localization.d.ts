import type { SoraReader } from "./sora_runtime.js";


export interface Localization {
    key: string;
    zhCn: string;
    enUs: string;
    note: string | undefined;
}

export declare function decodeLocalization(reader: SoraReader): Localization;
