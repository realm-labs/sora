import {
    SoraBundle,
    decodeIndex,
    decodeMapTable,
    decodeUniqueIndex,
    requireSingletonTable,
} from "./sora_runtime.js";

import type { Item } from "./item.js";
import { decodeItem } from "./item.js";

import type { Skill } from "./skill.js";
import { decodeSkill } from "./skill.js";

import type { Quest } from "./quest.js";
import { decodeQuest } from "./quest.js";

import type { QuestReward } from "./quest_reward.js";
import { decodeQuestReward } from "./quest_reward.js";

import type { GameSettings } from "./game_settings.js";
import { decodeGameSettings } from "./game_settings.js";

import type { Localization } from "./localization.js";
import { decodeLocalization } from "./localization.js";

import type { LevelExp } from "./level_exp.js";
import { decodeLevelExp } from "./level_exp.js";

import type { Character } from "./character.js";
import { decodeCharacter } from "./character.js";

import type { CharacterSkill } from "./character_skill.js";
import { decodeCharacterSkill } from "./character_skill.js";

import type { Buff } from "./buff.js";
import { decodeBuff } from "./buff.js";

import type { DropGroup } from "./drop_group.js";
import { decodeDropGroup } from "./drop_group.js";

import type { DropEntry } from "./drop_entry.js";
import { decodeDropEntry } from "./drop_entry.js";

import type { Monster } from "./monster.js";
import { decodeMonster } from "./monster.js";

import type { Stage } from "./stage.js";
import { decodeStage } from "./stage.js";

import type { StageReward } from "./stage_reward.js";
import { decodeStageReward } from "./stage_reward.js";

import type { Dungeon } from "./dungeon.js";
import { decodeDungeon } from "./dungeon.js";

import type { Shop } from "./shop.js";
import { decodeShop } from "./shop.js";

import type { ShopItem } from "./shop_item.js";
import { decodeShopItem } from "./shop_item.js";

import type { Recipe } from "./recipe.js";
import { decodeRecipe } from "./recipe.js";

import type { GachaPool } from "./gacha_pool.js";
import { decodeGachaPool } from "./gacha_pool.js";

import type { GachaItem } from "./gacha_item.js";
import { decodeGachaItem } from "./gacha_item.js";

import type { EquipmentSet } from "./equipment_set.js";
import { decodeEquipmentSet } from "./equipment_set.js";

import type { Achievement } from "./achievement.js";
import { decodeAchievement } from "./achievement.js";

import type { VipLevel } from "./vip_level.js";
import { decodeVipLevel } from "./vip_level.js";

import type { MailTemplate } from "./mail_template.js";
import { decodeMailTemplate } from "./mail_template.js";

import type { MailReward } from "./mail_reward.js";
import { decodeMailReward } from "./mail_reward.js";

import type { Dialogue } from "./dialogue.js";
import { decodeDialogue } from "./dialogue.js";

import type { EventRule } from "./event_rule.js";
import { decodeEventRule } from "./event_rule.js";


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
export class ItemTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Item>,
        private readonly _by_name: Map<string, Item>,
        private readonly _by_item_type: Map<ItemType, Item[]>,
    ) {}

    static decode(rows: Item[]): ItemTable {
        return new ItemTable(
            decodeMapTable(rows, (row) => row.id),
            decodeUniqueIndex(rows, (row) => row.name),
            decodeIndex(rows, (row) => row.itemType),
        );
    }

    name(): string {
        return "Item";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Item | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Item> {
        return this._rows;
    }
    getByName(name: string): Item | undefined {
        return this._by_name.get(name);
    }
    findByItemType(itemType: ItemType): Item[] {
        return this._by_item_type.get(itemType) ?? [];
    }
}
export class SkillTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Skill>,
    ) {}

    static decode(rows: Skill[]): SkillTable {
        return new SkillTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Skill";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Skill | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Skill> {
        return this._rows;
    }
}
export class QuestTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Quest>,
    ) {}

    static decode(rows: Quest[]): QuestTable {
        return new QuestTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Quest";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Quest | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Quest> {
        return this._rows;
    }
}
export class QuestRewardTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: QuestReward[],
    ) {}

    static decode(rows: QuestReward[]): QuestRewardTable {
        return new QuestRewardTable(
            rows,
        );
    }

    name(): string {
        return "QuestReward";
    }

    mode(): string {
        return "list";
    }

    key(): string | undefined {
        return undefined;
    }

    len(): number {
        return this._rows.length;
    }
    rows(): readonly QuestReward[] {
        return this._rows;
    }
}
export class GameSettingsTable implements SoraConfigTable {
    private constructor(
        private readonly _row: GameSettings,
    ) {}

    static decode(rows: GameSettings[]): GameSettingsTable {
        return new GameSettingsTable(
            requireSingletonTable(rows, "GameSettings"),
        );
    }

    name(): string {
        return "GameSettings";
    }

    mode(): string {
        return "singleton";
    }

    key(): string | undefined {
        return undefined;
    }

    len(): number {
        return 1;
    }
    row(): GameSettings {
        return this._row;
    }
}
export class LocalizationTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<string, Localization>,
    ) {}

    static decode(rows: Localization[]): LocalizationTable {
        return new LocalizationTable(
            decodeMapTable(rows, (row) => row.key),
        );
    }

    name(): string {
        return "Localization";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "key";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: string): Localization | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<string, Localization> {
        return this._rows;
    }
}
export class LevelExpTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, LevelExp>,
    ) {}

    static decode(rows: LevelExp[]): LevelExpTable {
        return new LevelExpTable(
            decodeMapTable(rows, (row) => row.level),
        );
    }

    name(): string {
        return "LevelExp";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "level";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): LevelExp | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, LevelExp> {
        return this._rows;
    }
}
export class CharacterTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Character>,
    ) {}

    static decode(rows: Character[]): CharacterTable {
        return new CharacterTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Character";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Character | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Character> {
        return this._rows;
    }
}
export class CharacterSkillTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: CharacterSkill[],
    ) {}

    static decode(rows: CharacterSkill[]): CharacterSkillTable {
        return new CharacterSkillTable(
            rows,
        );
    }

    name(): string {
        return "CharacterSkill";
    }

    mode(): string {
        return "list";
    }

    key(): string | undefined {
        return undefined;
    }

    len(): number {
        return this._rows.length;
    }
    rows(): readonly CharacterSkill[] {
        return this._rows;
    }
}
export class BuffTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Buff>,
    ) {}

    static decode(rows: Buff[]): BuffTable {
        return new BuffTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Buff";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Buff | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Buff> {
        return this._rows;
    }
}
export class DropGroupTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, DropGroup>,
    ) {}

    static decode(rows: DropGroup[]): DropGroupTable {
        return new DropGroupTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "DropGroup";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): DropGroup | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, DropGroup> {
        return this._rows;
    }
}
export class DropEntryTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: DropEntry[],
    ) {}

    static decode(rows: DropEntry[]): DropEntryTable {
        return new DropEntryTable(
            rows,
        );
    }

    name(): string {
        return "DropEntry";
    }

    mode(): string {
        return "list";
    }

    key(): string | undefined {
        return undefined;
    }

    len(): number {
        return this._rows.length;
    }
    rows(): readonly DropEntry[] {
        return this._rows;
    }
}
export class MonsterTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Monster>,
    ) {}

    static decode(rows: Monster[]): MonsterTable {
        return new MonsterTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Monster";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Monster | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Monster> {
        return this._rows;
    }
}
export class StageTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Stage>,
    ) {}

    static decode(rows: Stage[]): StageTable {
        return new StageTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Stage";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Stage | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Stage> {
        return this._rows;
    }
}
export class StageRewardTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: StageReward[],
    ) {}

    static decode(rows: StageReward[]): StageRewardTable {
        return new StageRewardTable(
            rows,
        );
    }

    name(): string {
        return "StageReward";
    }

    mode(): string {
        return "list";
    }

    key(): string | undefined {
        return undefined;
    }

    len(): number {
        return this._rows.length;
    }
    rows(): readonly StageReward[] {
        return this._rows;
    }
}
export class DungeonTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Dungeon>,
    ) {}

    static decode(rows: Dungeon[]): DungeonTable {
        return new DungeonTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Dungeon";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Dungeon | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Dungeon> {
        return this._rows;
    }
}
export class ShopTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Shop>,
    ) {}

    static decode(rows: Shop[]): ShopTable {
        return new ShopTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Shop";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Shop | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Shop> {
        return this._rows;
    }
}
export class ShopItemTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: ShopItem[],
    ) {}

    static decode(rows: ShopItem[]): ShopItemTable {
        return new ShopItemTable(
            rows,
        );
    }

    name(): string {
        return "ShopItem";
    }

    mode(): string {
        return "list";
    }

    key(): string | undefined {
        return undefined;
    }

    len(): number {
        return this._rows.length;
    }
    rows(): readonly ShopItem[] {
        return this._rows;
    }
}
export class RecipeTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Recipe>,
    ) {}

    static decode(rows: Recipe[]): RecipeTable {
        return new RecipeTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Recipe";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Recipe | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Recipe> {
        return this._rows;
    }
}
export class GachaPoolTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, GachaPool>,
    ) {}

    static decode(rows: GachaPool[]): GachaPoolTable {
        return new GachaPoolTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "GachaPool";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): GachaPool | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, GachaPool> {
        return this._rows;
    }
}
export class GachaItemTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: GachaItem[],
    ) {}

    static decode(rows: GachaItem[]): GachaItemTable {
        return new GachaItemTable(
            rows,
        );
    }

    name(): string {
        return "GachaItem";
    }

    mode(): string {
        return "list";
    }

    key(): string | undefined {
        return undefined;
    }

    len(): number {
        return this._rows.length;
    }
    rows(): readonly GachaItem[] {
        return this._rows;
    }
}
export class EquipmentSetTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, EquipmentSet>,
    ) {}

    static decode(rows: EquipmentSet[]): EquipmentSetTable {
        return new EquipmentSetTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "EquipmentSet";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): EquipmentSet | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, EquipmentSet> {
        return this._rows;
    }
}
export class AchievementTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Achievement>,
    ) {}

    static decode(rows: Achievement[]): AchievementTable {
        return new AchievementTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Achievement";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Achievement | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Achievement> {
        return this._rows;
    }
}
export class VipLevelTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, VipLevel>,
    ) {}

    static decode(rows: VipLevel[]): VipLevelTable {
        return new VipLevelTable(
            decodeMapTable(rows, (row) => row.level),
        );
    }

    name(): string {
        return "VipLevel";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "level";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): VipLevel | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, VipLevel> {
        return this._rows;
    }
}
export class MailTemplateTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, MailTemplate>,
    ) {}

    static decode(rows: MailTemplate[]): MailTemplateTable {
        return new MailTemplateTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "MailTemplate";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): MailTemplate | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, MailTemplate> {
        return this._rows;
    }
}
export class MailRewardTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: MailReward[],
    ) {}

    static decode(rows: MailReward[]): MailRewardTable {
        return new MailRewardTable(
            rows,
        );
    }

    name(): string {
        return "MailReward";
    }

    mode(): string {
        return "list";
    }

    key(): string | undefined {
        return undefined;
    }

    len(): number {
        return this._rows.length;
    }
    rows(): readonly MailReward[] {
        return this._rows;
    }
}
export class DialogueTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, Dialogue>,
    ) {}

    static decode(rows: Dialogue[]): DialogueTable {
        return new DialogueTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "Dialogue";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): Dialogue | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, Dialogue> {
        return this._rows;
    }
}
export class EventRuleTable implements SoraConfigTable {
    private constructor(
        private readonly _rows: Map<number, EventRule>,
    ) {}

    static decode(rows: EventRule[]): EventRuleTable {
        return new EventRuleTable(
            decodeMapTable(rows, (row) => row.id),
        );
    }

    name(): string {
        return "EventRule";
    }

    mode(): string {
        return "map";
    }

    key(): string | undefined {
        return "id";
    }

    len(): number {
        return this._rows.size;
    }
    get(key: number): EventRule | undefined {
        return this._rows.get(key);
    }

    rows(): ReadonlyMap<number, EventRule> {
        return this._rows;
    }
}
export class SoraConfig {
    private constructor(
        private readonly _item: ItemTable,
        private readonly _skill: SkillTable,
        private readonly _quest: QuestTable,
        private readonly _questReward: QuestRewardTable,
        private readonly _gameSettings: GameSettingsTable,
        private readonly _localization: LocalizationTable,
        private readonly _levelExp: LevelExpTable,
        private readonly _character: CharacterTable,
        private readonly _characterSkill: CharacterSkillTable,
        private readonly _buff: BuffTable,
        private readonly _dropGroup: DropGroupTable,
        private readonly _dropEntry: DropEntryTable,
        private readonly _monster: MonsterTable,
        private readonly _stage: StageTable,
        private readonly _stageReward: StageRewardTable,
        private readonly _dungeon: DungeonTable,
        private readonly _shop: ShopTable,
        private readonly _shopItem: ShopItemTable,
        private readonly _recipe: RecipeTable,
        private readonly _gachaPool: GachaPoolTable,
        private readonly _gachaItem: GachaItemTable,
        private readonly _equipmentSet: EquipmentSetTable,
        private readonly _achievement: AchievementTable,
        private readonly _vipLevel: VipLevelTable,
        private readonly _mailTemplate: MailTemplateTable,
        private readonly _mailReward: MailRewardTable,
        private readonly _dialogue: DialogueTable,
        private readonly _eventRule: EventRuleTable,
    ) {}

    static fromBytes(bytes: Uint8Array | ArrayBuffer): SoraConfig {
        const bundle = SoraBundle.parse(bytes);
        return new SoraConfig(
            ItemTable.decode(bundle.decodeTable("Item", decodeItem)),
            SkillTable.decode(bundle.decodeTable("Skill", decodeSkill)),
            QuestTable.decode(bundle.decodeTable("Quest", decodeQuest)),
            QuestRewardTable.decode(bundle.decodeTable("QuestReward", decodeQuestReward)),
            GameSettingsTable.decode(bundle.decodeTable("GameSettings", decodeGameSettings)),
            LocalizationTable.decode(bundle.decodeTable("Localization", decodeLocalization)),
            LevelExpTable.decode(bundle.decodeTable("LevelExp", decodeLevelExp)),
            CharacterTable.decode(bundle.decodeTable("Character", decodeCharacter)),
            CharacterSkillTable.decode(bundle.decodeTable("CharacterSkill", decodeCharacterSkill)),
            BuffTable.decode(bundle.decodeTable("Buff", decodeBuff)),
            DropGroupTable.decode(bundle.decodeTable("DropGroup", decodeDropGroup)),
            DropEntryTable.decode(bundle.decodeTable("DropEntry", decodeDropEntry)),
            MonsterTable.decode(bundle.decodeTable("Monster", decodeMonster)),
            StageTable.decode(bundle.decodeTable("Stage", decodeStage)),
            StageRewardTable.decode(bundle.decodeTable("StageReward", decodeStageReward)),
            DungeonTable.decode(bundle.decodeTable("Dungeon", decodeDungeon)),
            ShopTable.decode(bundle.decodeTable("Shop", decodeShop)),
            ShopItemTable.decode(bundle.decodeTable("ShopItem", decodeShopItem)),
            RecipeTable.decode(bundle.decodeTable("Recipe", decodeRecipe)),
            GachaPoolTable.decode(bundle.decodeTable("GachaPool", decodeGachaPool)),
            GachaItemTable.decode(bundle.decodeTable("GachaItem", decodeGachaItem)),
            EquipmentSetTable.decode(bundle.decodeTable("EquipmentSet", decodeEquipmentSet)),
            AchievementTable.decode(bundle.decodeTable("Achievement", decodeAchievement)),
            VipLevelTable.decode(bundle.decodeTable("VipLevel", decodeVipLevel)),
            MailTemplateTable.decode(bundle.decodeTable("MailTemplate", decodeMailTemplate)),
            MailRewardTable.decode(bundle.decodeTable("MailReward", decodeMailReward)),
            DialogueTable.decode(bundle.decodeTable("Dialogue", decodeDialogue)),
            EventRuleTable.decode(bundle.decodeTable("EventRule", decodeEventRule)),
        );
    }

    tables(): SoraConfigTable[] {
        return [
            this._item,
            this._skill,
            this._quest,
            this._questReward,
            this._gameSettings,
            this._localization,
            this._levelExp,
            this._character,
            this._characterSkill,
            this._buff,
            this._dropGroup,
            this._dropEntry,
            this._monster,
            this._stage,
            this._stageReward,
            this._dungeon,
            this._shop,
            this._shopItem,
            this._recipe,
            this._gachaPool,
            this._gachaItem,
            this._equipmentSet,
            this._achievement,
            this._vipLevel,
            this._mailTemplate,
            this._mailReward,
            this._dialogue,
            this._eventRule,
        ];
    }
    item(): ItemTable {
        return this._item;
    }
    skill(): SkillTable {
        return this._skill;
    }
    quest(): QuestTable {
        return this._quest;
    }
    questReward(): QuestRewardTable {
        return this._questReward;
    }
    gameSettings(): GameSettingsTable {
        return this._gameSettings;
    }
    localization(): LocalizationTable {
        return this._localization;
    }
    levelExp(): LevelExpTable {
        return this._levelExp;
    }
    character(): CharacterTable {
        return this._character;
    }
    characterSkill(): CharacterSkillTable {
        return this._characterSkill;
    }
    buff(): BuffTable {
        return this._buff;
    }
    dropGroup(): DropGroupTable {
        return this._dropGroup;
    }
    dropEntry(): DropEntryTable {
        return this._dropEntry;
    }
    monster(): MonsterTable {
        return this._monster;
    }
    stage(): StageTable {
        return this._stage;
    }
    stageReward(): StageRewardTable {
        return this._stageReward;
    }
    dungeon(): DungeonTable {
        return this._dungeon;
    }
    shop(): ShopTable {
        return this._shop;
    }
    shopItem(): ShopItemTable {
        return this._shopItem;
    }
    recipe(): RecipeTable {
        return this._recipe;
    }
    gachaPool(): GachaPoolTable {
        return this._gachaPool;
    }
    gachaItem(): GachaItemTable {
        return this._gachaItem;
    }
    equipmentSet(): EquipmentSetTable {
        return this._equipmentSet;
    }
    achievement(): AchievementTable {
        return this._achievement;
    }
    vipLevel(): VipLevelTable {
        return this._vipLevel;
    }
    mailTemplate(): MailTemplateTable {
        return this._mailTemplate;
    }
    mailReward(): MailRewardTable {
        return this._mailReward;
    }
    dialogue(): DialogueTable {
        return this._dialogue;
    }
    eventRule(): EventRuleTable {
        return this._eventRule;
    }
}
