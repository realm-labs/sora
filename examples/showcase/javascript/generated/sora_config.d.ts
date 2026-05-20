
import type { Item } from "./item.js";

import type { Skill } from "./skill.js";

import type { Quest } from "./quest.js";

import type { QuestReward } from "./quest_reward.js";

import type { GameSettings } from "./game_settings.js";

import type { Localization } from "./localization.js";

import type { LevelExp } from "./level_exp.js";

import type { Character } from "./character.js";

import type { CharacterSkill } from "./character_skill.js";

import type { Buff } from "./buff.js";

import type { DropGroup } from "./drop_group.js";

import type { DropEntry } from "./drop_entry.js";

import type { Monster } from "./monster.js";

import type { Stage } from "./stage.js";

import type { StageReward } from "./stage_reward.js";

import type { Dungeon } from "./dungeon.js";

import type { Shop } from "./shop.js";

import type { ShopItem } from "./shop_item.js";

import type { Recipe } from "./recipe.js";

import type { GachaPool } from "./gacha_pool.js";

import type { GachaItem } from "./gacha_item.js";

import type { EquipmentSet } from "./equipment_set.js";

import type { Achievement } from "./achievement.js";

import type { VipLevel } from "./vip_level.js";

import type { MailTemplate } from "./mail_template.js";

import type { MailReward } from "./mail_reward.js";

import type { Dialogue } from "./dialogue.js";

import type { EventRule } from "./event_rule.js";


import type { ItemType } from "./item_type.js";

import type { ResourceKind } from "./resource_kind.js";

import type { ElementType } from "./element_type.js";

import type { QuestType } from "./quest_type.js";

import type { Rarity } from "./rarity.js";

import type { StatType } from "./stat_type.js";

import type { MailType } from "./mail_type.js";


export interface SoraConfigTable {
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
}
export declare class ItemTable implements SoraConfigTable {
    static decode(rows: Item[]): ItemTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Item | undefined;
    rows(): ReadonlyMap<number, Item>;
    getByName(name: string): Item | undefined;
    findByItemType(itemType: ItemType): Item[];
}
export declare class SkillTable implements SoraConfigTable {
    static decode(rows: Skill[]): SkillTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Skill | undefined;
    rows(): ReadonlyMap<number, Skill>;
}
export declare class QuestTable implements SoraConfigTable {
    static decode(rows: Quest[]): QuestTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Quest | undefined;
    rows(): ReadonlyMap<number, Quest>;
}
export declare class QuestRewardTable implements SoraConfigTable {
    static decode(rows: QuestReward[]): QuestRewardTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    rows(): readonly QuestReward[];
}
export declare class GameSettingsTable implements SoraConfigTable {
    static decode(rows: GameSettings[]): GameSettingsTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    row(): GameSettings;
}
export declare class LocalizationTable implements SoraConfigTable {
    static decode(rows: Localization[]): LocalizationTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: string): Localization | undefined;
    rows(): ReadonlyMap<string, Localization>;
}
export declare class LevelExpTable implements SoraConfigTable {
    static decode(rows: LevelExp[]): LevelExpTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): LevelExp | undefined;
    rows(): ReadonlyMap<number, LevelExp>;
}
export declare class CharacterTable implements SoraConfigTable {
    static decode(rows: Character[]): CharacterTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Character | undefined;
    rows(): ReadonlyMap<number, Character>;
}
export declare class CharacterSkillTable implements SoraConfigTable {
    static decode(rows: CharacterSkill[]): CharacterSkillTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    rows(): readonly CharacterSkill[];
}
export declare class BuffTable implements SoraConfigTable {
    static decode(rows: Buff[]): BuffTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Buff | undefined;
    rows(): ReadonlyMap<number, Buff>;
}
export declare class DropGroupTable implements SoraConfigTable {
    static decode(rows: DropGroup[]): DropGroupTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): DropGroup | undefined;
    rows(): ReadonlyMap<number, DropGroup>;
}
export declare class DropEntryTable implements SoraConfigTable {
    static decode(rows: DropEntry[]): DropEntryTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    rows(): readonly DropEntry[];
}
export declare class MonsterTable implements SoraConfigTable {
    static decode(rows: Monster[]): MonsterTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Monster | undefined;
    rows(): ReadonlyMap<number, Monster>;
}
export declare class StageTable implements SoraConfigTable {
    static decode(rows: Stage[]): StageTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Stage | undefined;
    rows(): ReadonlyMap<number, Stage>;
}
export declare class StageRewardTable implements SoraConfigTable {
    static decode(rows: StageReward[]): StageRewardTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    rows(): readonly StageReward[];
}
export declare class DungeonTable implements SoraConfigTable {
    static decode(rows: Dungeon[]): DungeonTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Dungeon | undefined;
    rows(): ReadonlyMap<number, Dungeon>;
}
export declare class ShopTable implements SoraConfigTable {
    static decode(rows: Shop[]): ShopTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Shop | undefined;
    rows(): ReadonlyMap<number, Shop>;
}
export declare class ShopItemTable implements SoraConfigTable {
    static decode(rows: ShopItem[]): ShopItemTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    rows(): readonly ShopItem[];
}
export declare class RecipeTable implements SoraConfigTable {
    static decode(rows: Recipe[]): RecipeTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Recipe | undefined;
    rows(): ReadonlyMap<number, Recipe>;
}
export declare class GachaPoolTable implements SoraConfigTable {
    static decode(rows: GachaPool[]): GachaPoolTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): GachaPool | undefined;
    rows(): ReadonlyMap<number, GachaPool>;
}
export declare class GachaItemTable implements SoraConfigTable {
    static decode(rows: GachaItem[]): GachaItemTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    rows(): readonly GachaItem[];
}
export declare class EquipmentSetTable implements SoraConfigTable {
    static decode(rows: EquipmentSet[]): EquipmentSetTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): EquipmentSet | undefined;
    rows(): ReadonlyMap<number, EquipmentSet>;
}
export declare class AchievementTable implements SoraConfigTable {
    static decode(rows: Achievement[]): AchievementTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Achievement | undefined;
    rows(): ReadonlyMap<number, Achievement>;
}
export declare class VipLevelTable implements SoraConfigTable {
    static decode(rows: VipLevel[]): VipLevelTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): VipLevel | undefined;
    rows(): ReadonlyMap<number, VipLevel>;
}
export declare class MailTemplateTable implements SoraConfigTable {
    static decode(rows: MailTemplate[]): MailTemplateTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): MailTemplate | undefined;
    rows(): ReadonlyMap<number, MailTemplate>;
}
export declare class MailRewardTable implements SoraConfigTable {
    static decode(rows: MailReward[]): MailRewardTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    rows(): readonly MailReward[];
}
export declare class DialogueTable implements SoraConfigTable {
    static decode(rows: Dialogue[]): DialogueTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): Dialogue | undefined;
    rows(): ReadonlyMap<number, Dialogue>;
}
export declare class EventRuleTable implements SoraConfigTable {
    static decode(rows: EventRule[]): EventRuleTable;
    name(): string;
    mode(): string;
    key(): string | undefined;
    len(): number;
    get(key: number): EventRule | undefined;
    rows(): ReadonlyMap<number, EventRule>;
}
export declare class SoraConfig {
    static fromBytes(bytes: Uint8Array | ArrayBuffer): SoraConfig;
    tables(): SoraConfigTable[];
    item(): ItemTable;
    skill(): SkillTable;
    quest(): QuestTable;
    questReward(): QuestRewardTable;
    gameSettings(): GameSettingsTable;
    localization(): LocalizationTable;
    levelExp(): LevelExpTable;
    character(): CharacterTable;
    characterSkill(): CharacterSkillTable;
    buff(): BuffTable;
    dropGroup(): DropGroupTable;
    dropEntry(): DropEntryTable;
    monster(): MonsterTable;
    stage(): StageTable;
    stageReward(): StageRewardTable;
    dungeon(): DungeonTable;
    shop(): ShopTable;
    shopItem(): ShopItemTable;
    recipe(): RecipeTable;
    gachaPool(): GachaPoolTable;
    gachaItem(): GachaItemTable;
    equipmentSet(): EquipmentSetTable;
    achievement(): AchievementTable;
    vipLevel(): VipLevelTable;
    mailTemplate(): MailTemplateTable;
    mailReward(): MailRewardTable;
    dialogue(): DialogueTable;
    eventRule(): EventRuleTable;
}
