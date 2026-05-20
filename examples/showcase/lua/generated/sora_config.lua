local Runtime = require("generated.sora_runtime")
local Item = require("generated.item")
local Skill = require("generated.skill")
local Quest = require("generated.quest")
local QuestReward = require("generated.quest_reward")
local GameSettings = require("generated.game_settings")
local Localization = require("generated.localization")
local LevelExp = require("generated.level_exp")
local Character = require("generated.character")
local CharacterSkill = require("generated.character_skill")
local Buff = require("generated.buff")
local DropGroup = require("generated.drop_group")
local DropEntry = require("generated.drop_entry")
local Monster = require("generated.monster")
local Stage = require("generated.stage")
local StageReward = require("generated.stage_reward")
local Dungeon = require("generated.dungeon")
local Shop = require("generated.shop")
local ShopItem = require("generated.shop_item")
local Recipe = require("generated.recipe")
local GachaPool = require("generated.gacha_pool")
local GachaItem = require("generated.gacha_item")
local EquipmentSet = require("generated.equipment_set")
local Achievement = require("generated.achievement")
local VipLevel = require("generated.vip_level")
local MailTemplate = require("generated.mail_template")
local MailReward = require("generated.mail_reward")
local Dialogue = require("generated.dialogue")
local EventRule = require("generated.event_rule")
---@class ItemTable
---@field private _rows table<integer, Item>
---@field private _by_name table<string, Item>
---@field private _by_item_type table<ItemType, Item[]>
local ItemTable = {}
ItemTable.__index = ItemTable

---@param rows Item[]
---@return ItemTable
function ItemTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
        _by_name = Runtime.decode_unique_index(rows, function(row) return row.name end),
        _by_item_type = Runtime.decode_index(rows, function(row) return row.itemType end),
    }, ItemTable)
end

---@return string
function ItemTable:name()
    return "Item"
end

---@return string
function ItemTable:mode()
    return "map"
end

---@return string?
function ItemTable:key()
    return "id"
end

---@return integer
function ItemTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Item?
function ItemTable:get(key)
    return self._rows[key]
end

---@return table<integer, Item>
function ItemTable:rows()
    return self._rows
end
---@param name string
---@return Item?
function ItemTable:get_by_name(name)
    return self._by_name[name]
end
---@param itemType ItemType
---@return Item[]
function ItemTable:find_by_item_type(itemType)
    return self._by_item_type[itemType] or {}
end
---@class SkillTable
---@field private _rows table<integer, Skill>
local SkillTable = {}
SkillTable.__index = SkillTable

---@param rows Skill[]
---@return SkillTable
function SkillTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, SkillTable)
end

---@return string
function SkillTable:name()
    return "Skill"
end

---@return string
function SkillTable:mode()
    return "map"
end

---@return string?
function SkillTable:key()
    return "id"
end

---@return integer
function SkillTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Skill?
function SkillTable:get(key)
    return self._rows[key]
end

---@return table<integer, Skill>
function SkillTable:rows()
    return self._rows
end
---@class QuestTable
---@field private _rows table<integer, Quest>
local QuestTable = {}
QuestTable.__index = QuestTable

---@param rows Quest[]
---@return QuestTable
function QuestTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, QuestTable)
end

---@return string
function QuestTable:name()
    return "Quest"
end

---@return string
function QuestTable:mode()
    return "map"
end

---@return string?
function QuestTable:key()
    return "id"
end

---@return integer
function QuestTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Quest?
function QuestTable:get(key)
    return self._rows[key]
end

---@return table<integer, Quest>
function QuestTable:rows()
    return self._rows
end
---@class QuestRewardTable
---@field private _rows QuestReward[]
local QuestRewardTable = {}
QuestRewardTable.__index = QuestRewardTable

---@param rows QuestReward[]
---@return QuestRewardTable
function QuestRewardTable.decode(rows)
    return setmetatable({
        _rows = rows,
    }, QuestRewardTable)
end

---@return string
function QuestRewardTable:name()
    return "QuestReward"
end

---@return string
function QuestRewardTable:mode()
    return "list"
end

---@return string?
function QuestRewardTable:key()
    return nil
end

---@return integer
function QuestRewardTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@return QuestReward[]
function QuestRewardTable:rows()
    return self._rows
end
---@class GameSettingsTable
---@field private _row GameSettings
local GameSettingsTable = {}
GameSettingsTable.__index = GameSettingsTable

---@param rows GameSettings[]
---@return GameSettingsTable
function GameSettingsTable.decode(rows)
    return setmetatable({
        _row = Runtime.require_singleton_table(rows, "GameSettings"),
    }, GameSettingsTable)
end

---@return string
function GameSettingsTable:name()
    return "GameSettings"
end

---@return string
function GameSettingsTable:mode()
    return "singleton"
end

---@return string?
function GameSettingsTable:key()
    return nil
end

---@return integer
function GameSettingsTable:len()
    return 1
end
---@return GameSettings
function GameSettingsTable:row()
    return self._row
end
---@class LocalizationTable
---@field private _rows table<string, Localization>
local LocalizationTable = {}
LocalizationTable.__index = LocalizationTable

---@param rows Localization[]
---@return LocalizationTable
function LocalizationTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.key end),
    }, LocalizationTable)
end

---@return string
function LocalizationTable:name()
    return "Localization"
end

---@return string
function LocalizationTable:mode()
    return "map"
end

---@return string?
function LocalizationTable:key()
    return "key"
end

---@return integer
function LocalizationTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key string
---@return Localization?
function LocalizationTable:get(key)
    return self._rows[key]
end

---@return table<string, Localization>
function LocalizationTable:rows()
    return self._rows
end
---@class LevelExpTable
---@field private _rows table<integer, LevelExp>
local LevelExpTable = {}
LevelExpTable.__index = LevelExpTable

---@param rows LevelExp[]
---@return LevelExpTable
function LevelExpTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.level end),
    }, LevelExpTable)
end

---@return string
function LevelExpTable:name()
    return "LevelExp"
end

---@return string
function LevelExpTable:mode()
    return "map"
end

---@return string?
function LevelExpTable:key()
    return "level"
end

---@return integer
function LevelExpTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return LevelExp?
function LevelExpTable:get(key)
    return self._rows[key]
end

---@return table<integer, LevelExp>
function LevelExpTable:rows()
    return self._rows
end
---@class CharacterTable
---@field private _rows table<integer, Character>
local CharacterTable = {}
CharacterTable.__index = CharacterTable

---@param rows Character[]
---@return CharacterTable
function CharacterTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, CharacterTable)
end

---@return string
function CharacterTable:name()
    return "Character"
end

---@return string
function CharacterTable:mode()
    return "map"
end

---@return string?
function CharacterTable:key()
    return "id"
end

---@return integer
function CharacterTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Character?
function CharacterTable:get(key)
    return self._rows[key]
end

---@return table<integer, Character>
function CharacterTable:rows()
    return self._rows
end
---@class CharacterSkillTable
---@field private _rows CharacterSkill[]
local CharacterSkillTable = {}
CharacterSkillTable.__index = CharacterSkillTable

---@param rows CharacterSkill[]
---@return CharacterSkillTable
function CharacterSkillTable.decode(rows)
    return setmetatable({
        _rows = rows,
    }, CharacterSkillTable)
end

---@return string
function CharacterSkillTable:name()
    return "CharacterSkill"
end

---@return string
function CharacterSkillTable:mode()
    return "list"
end

---@return string?
function CharacterSkillTable:key()
    return nil
end

---@return integer
function CharacterSkillTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@return CharacterSkill[]
function CharacterSkillTable:rows()
    return self._rows
end
---@class BuffTable
---@field private _rows table<integer, Buff>
local BuffTable = {}
BuffTable.__index = BuffTable

---@param rows Buff[]
---@return BuffTable
function BuffTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, BuffTable)
end

---@return string
function BuffTable:name()
    return "Buff"
end

---@return string
function BuffTable:mode()
    return "map"
end

---@return string?
function BuffTable:key()
    return "id"
end

---@return integer
function BuffTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Buff?
function BuffTable:get(key)
    return self._rows[key]
end

---@return table<integer, Buff>
function BuffTable:rows()
    return self._rows
end
---@class DropGroupTable
---@field private _rows table<integer, DropGroup>
local DropGroupTable = {}
DropGroupTable.__index = DropGroupTable

---@param rows DropGroup[]
---@return DropGroupTable
function DropGroupTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, DropGroupTable)
end

---@return string
function DropGroupTable:name()
    return "DropGroup"
end

---@return string
function DropGroupTable:mode()
    return "map"
end

---@return string?
function DropGroupTable:key()
    return "id"
end

---@return integer
function DropGroupTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return DropGroup?
function DropGroupTable:get(key)
    return self._rows[key]
end

---@return table<integer, DropGroup>
function DropGroupTable:rows()
    return self._rows
end
---@class DropEntryTable
---@field private _rows DropEntry[]
local DropEntryTable = {}
DropEntryTable.__index = DropEntryTable

---@param rows DropEntry[]
---@return DropEntryTable
function DropEntryTable.decode(rows)
    return setmetatable({
        _rows = rows,
    }, DropEntryTable)
end

---@return string
function DropEntryTable:name()
    return "DropEntry"
end

---@return string
function DropEntryTable:mode()
    return "list"
end

---@return string?
function DropEntryTable:key()
    return nil
end

---@return integer
function DropEntryTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@return DropEntry[]
function DropEntryTable:rows()
    return self._rows
end
---@class MonsterTable
---@field private _rows table<integer, Monster>
local MonsterTable = {}
MonsterTable.__index = MonsterTable

---@param rows Monster[]
---@return MonsterTable
function MonsterTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, MonsterTable)
end

---@return string
function MonsterTable:name()
    return "Monster"
end

---@return string
function MonsterTable:mode()
    return "map"
end

---@return string?
function MonsterTable:key()
    return "id"
end

---@return integer
function MonsterTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Monster?
function MonsterTable:get(key)
    return self._rows[key]
end

---@return table<integer, Monster>
function MonsterTable:rows()
    return self._rows
end
---@class StageTable
---@field private _rows table<integer, Stage>
local StageTable = {}
StageTable.__index = StageTable

---@param rows Stage[]
---@return StageTable
function StageTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, StageTable)
end

---@return string
function StageTable:name()
    return "Stage"
end

---@return string
function StageTable:mode()
    return "map"
end

---@return string?
function StageTable:key()
    return "id"
end

---@return integer
function StageTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Stage?
function StageTable:get(key)
    return self._rows[key]
end

---@return table<integer, Stage>
function StageTable:rows()
    return self._rows
end
---@class StageRewardTable
---@field private _rows StageReward[]
local StageRewardTable = {}
StageRewardTable.__index = StageRewardTable

---@param rows StageReward[]
---@return StageRewardTable
function StageRewardTable.decode(rows)
    return setmetatable({
        _rows = rows,
    }, StageRewardTable)
end

---@return string
function StageRewardTable:name()
    return "StageReward"
end

---@return string
function StageRewardTable:mode()
    return "list"
end

---@return string?
function StageRewardTable:key()
    return nil
end

---@return integer
function StageRewardTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@return StageReward[]
function StageRewardTable:rows()
    return self._rows
end
---@class DungeonTable
---@field private _rows table<integer, Dungeon>
local DungeonTable = {}
DungeonTable.__index = DungeonTable

---@param rows Dungeon[]
---@return DungeonTable
function DungeonTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, DungeonTable)
end

---@return string
function DungeonTable:name()
    return "Dungeon"
end

---@return string
function DungeonTable:mode()
    return "map"
end

---@return string?
function DungeonTable:key()
    return "id"
end

---@return integer
function DungeonTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Dungeon?
function DungeonTable:get(key)
    return self._rows[key]
end

---@return table<integer, Dungeon>
function DungeonTable:rows()
    return self._rows
end
---@class ShopTable
---@field private _rows table<integer, Shop>
local ShopTable = {}
ShopTable.__index = ShopTable

---@param rows Shop[]
---@return ShopTable
function ShopTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, ShopTable)
end

---@return string
function ShopTable:name()
    return "Shop"
end

---@return string
function ShopTable:mode()
    return "map"
end

---@return string?
function ShopTable:key()
    return "id"
end

---@return integer
function ShopTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Shop?
function ShopTable:get(key)
    return self._rows[key]
end

---@return table<integer, Shop>
function ShopTable:rows()
    return self._rows
end
---@class ShopItemTable
---@field private _rows ShopItem[]
local ShopItemTable = {}
ShopItemTable.__index = ShopItemTable

---@param rows ShopItem[]
---@return ShopItemTable
function ShopItemTable.decode(rows)
    return setmetatable({
        _rows = rows,
    }, ShopItemTable)
end

---@return string
function ShopItemTable:name()
    return "ShopItem"
end

---@return string
function ShopItemTable:mode()
    return "list"
end

---@return string?
function ShopItemTable:key()
    return nil
end

---@return integer
function ShopItemTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@return ShopItem[]
function ShopItemTable:rows()
    return self._rows
end
---@class RecipeTable
---@field private _rows table<integer, Recipe>
local RecipeTable = {}
RecipeTable.__index = RecipeTable

---@param rows Recipe[]
---@return RecipeTable
function RecipeTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, RecipeTable)
end

---@return string
function RecipeTable:name()
    return "Recipe"
end

---@return string
function RecipeTable:mode()
    return "map"
end

---@return string?
function RecipeTable:key()
    return "id"
end

---@return integer
function RecipeTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Recipe?
function RecipeTable:get(key)
    return self._rows[key]
end

---@return table<integer, Recipe>
function RecipeTable:rows()
    return self._rows
end
---@class GachaPoolTable
---@field private _rows table<integer, GachaPool>
local GachaPoolTable = {}
GachaPoolTable.__index = GachaPoolTable

---@param rows GachaPool[]
---@return GachaPoolTable
function GachaPoolTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, GachaPoolTable)
end

---@return string
function GachaPoolTable:name()
    return "GachaPool"
end

---@return string
function GachaPoolTable:mode()
    return "map"
end

---@return string?
function GachaPoolTable:key()
    return "id"
end

---@return integer
function GachaPoolTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return GachaPool?
function GachaPoolTable:get(key)
    return self._rows[key]
end

---@return table<integer, GachaPool>
function GachaPoolTable:rows()
    return self._rows
end
---@class GachaItemTable
---@field private _rows GachaItem[]
local GachaItemTable = {}
GachaItemTable.__index = GachaItemTable

---@param rows GachaItem[]
---@return GachaItemTable
function GachaItemTable.decode(rows)
    return setmetatable({
        _rows = rows,
    }, GachaItemTable)
end

---@return string
function GachaItemTable:name()
    return "GachaItem"
end

---@return string
function GachaItemTable:mode()
    return "list"
end

---@return string?
function GachaItemTable:key()
    return nil
end

---@return integer
function GachaItemTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@return GachaItem[]
function GachaItemTable:rows()
    return self._rows
end
---@class EquipmentSetTable
---@field private _rows table<integer, EquipmentSet>
local EquipmentSetTable = {}
EquipmentSetTable.__index = EquipmentSetTable

---@param rows EquipmentSet[]
---@return EquipmentSetTable
function EquipmentSetTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, EquipmentSetTable)
end

---@return string
function EquipmentSetTable:name()
    return "EquipmentSet"
end

---@return string
function EquipmentSetTable:mode()
    return "map"
end

---@return string?
function EquipmentSetTable:key()
    return "id"
end

---@return integer
function EquipmentSetTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return EquipmentSet?
function EquipmentSetTable:get(key)
    return self._rows[key]
end

---@return table<integer, EquipmentSet>
function EquipmentSetTable:rows()
    return self._rows
end
---@class AchievementTable
---@field private _rows table<integer, Achievement>
local AchievementTable = {}
AchievementTable.__index = AchievementTable

---@param rows Achievement[]
---@return AchievementTable
function AchievementTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, AchievementTable)
end

---@return string
function AchievementTable:name()
    return "Achievement"
end

---@return string
function AchievementTable:mode()
    return "map"
end

---@return string?
function AchievementTable:key()
    return "id"
end

---@return integer
function AchievementTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Achievement?
function AchievementTable:get(key)
    return self._rows[key]
end

---@return table<integer, Achievement>
function AchievementTable:rows()
    return self._rows
end
---@class VipLevelTable
---@field private _rows table<integer, VipLevel>
local VipLevelTable = {}
VipLevelTable.__index = VipLevelTable

---@param rows VipLevel[]
---@return VipLevelTable
function VipLevelTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.level end),
    }, VipLevelTable)
end

---@return string
function VipLevelTable:name()
    return "VipLevel"
end

---@return string
function VipLevelTable:mode()
    return "map"
end

---@return string?
function VipLevelTable:key()
    return "level"
end

---@return integer
function VipLevelTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return VipLevel?
function VipLevelTable:get(key)
    return self._rows[key]
end

---@return table<integer, VipLevel>
function VipLevelTable:rows()
    return self._rows
end
---@class MailTemplateTable
---@field private _rows table<integer, MailTemplate>
local MailTemplateTable = {}
MailTemplateTable.__index = MailTemplateTable

---@param rows MailTemplate[]
---@return MailTemplateTable
function MailTemplateTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, MailTemplateTable)
end

---@return string
function MailTemplateTable:name()
    return "MailTemplate"
end

---@return string
function MailTemplateTable:mode()
    return "map"
end

---@return string?
function MailTemplateTable:key()
    return "id"
end

---@return integer
function MailTemplateTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return MailTemplate?
function MailTemplateTable:get(key)
    return self._rows[key]
end

---@return table<integer, MailTemplate>
function MailTemplateTable:rows()
    return self._rows
end
---@class MailRewardTable
---@field private _rows MailReward[]
local MailRewardTable = {}
MailRewardTable.__index = MailRewardTable

---@param rows MailReward[]
---@return MailRewardTable
function MailRewardTable.decode(rows)
    return setmetatable({
        _rows = rows,
    }, MailRewardTable)
end

---@return string
function MailRewardTable:name()
    return "MailReward"
end

---@return string
function MailRewardTable:mode()
    return "list"
end

---@return string?
function MailRewardTable:key()
    return nil
end

---@return integer
function MailRewardTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@return MailReward[]
function MailRewardTable:rows()
    return self._rows
end
---@class DialogueTable
---@field private _rows table<integer, Dialogue>
local DialogueTable = {}
DialogueTable.__index = DialogueTable

---@param rows Dialogue[]
---@return DialogueTable
function DialogueTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, DialogueTable)
end

---@return string
function DialogueTable:name()
    return "Dialogue"
end

---@return string
function DialogueTable:mode()
    return "map"
end

---@return string?
function DialogueTable:key()
    return "id"
end

---@return integer
function DialogueTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return Dialogue?
function DialogueTable:get(key)
    return self._rows[key]
end

---@return table<integer, Dialogue>
function DialogueTable:rows()
    return self._rows
end
---@class EventRuleTable
---@field private _rows table<integer, EventRule>
local EventRuleTable = {}
EventRuleTable.__index = EventRuleTable

---@param rows EventRule[]
---@return EventRuleTable
function EventRuleTable.decode(rows)
    return setmetatable({
        _rows = Runtime.decode_map_table(rows, function(row) return row.id end),
    }, EventRuleTable)
end

---@return string
function EventRuleTable:name()
    return "EventRule"
end

---@return string
function EventRuleTable:mode()
    return "map"
end

---@return string?
function EventRuleTable:key()
    return "id"
end

---@return integer
function EventRuleTable:len()
    local count = 0
    for _ in pairs(self._rows) do
        count = count + 1
    end
    return count
end
---@param key integer
---@return EventRule?
function EventRuleTable:get(key)
    return self._rows[key]
end

---@return table<integer, EventRule>
function EventRuleTable:rows()
    return self._rows
end
---@class SoraConfig
---@field private _item ItemTable
---@field private _skill SkillTable
---@field private _quest QuestTable
---@field private _quest_reward QuestRewardTable
---@field private _game_settings GameSettingsTable
---@field private _localization LocalizationTable
---@field private _level_exp LevelExpTable
---@field private _character CharacterTable
---@field private _character_skill CharacterSkillTable
---@field private _buff BuffTable
---@field private _drop_group DropGroupTable
---@field private _drop_entry DropEntryTable
---@field private _monster MonsterTable
---@field private _stage StageTable
---@field private _stage_reward StageRewardTable
---@field private _dungeon DungeonTable
---@field private _shop ShopTable
---@field private _shop_item ShopItemTable
---@field private _recipe RecipeTable
---@field private _gacha_pool GachaPoolTable
---@field private _gacha_item GachaItemTable
---@field private _equipment_set EquipmentSetTable
---@field private _achievement AchievementTable
---@field private _vip_level VipLevelTable
---@field private _mail_template MailTemplateTable
---@field private _mail_reward MailRewardTable
---@field private _dialogue DialogueTable
---@field private _event_rule EventRuleTable
local SoraConfig = {}
SoraConfig.__index = SoraConfig

---@param bytes string
---@return SoraConfig
function SoraConfig.from_bytes(bytes)
    local bundle = Runtime.parse_bundle(bytes)
    return setmetatable({
        _item = ItemTable.decode(bundle:decode_table("Item", Item.decode)),
        _skill = SkillTable.decode(bundle:decode_table("Skill", Skill.decode)),
        _quest = QuestTable.decode(bundle:decode_table("Quest", Quest.decode)),
        _quest_reward = QuestRewardTable.decode(bundle:decode_table("QuestReward", QuestReward.decode)),
        _game_settings = GameSettingsTable.decode(bundle:decode_table("GameSettings", GameSettings.decode)),
        _localization = LocalizationTable.decode(bundle:decode_table("Localization", Localization.decode)),
        _level_exp = LevelExpTable.decode(bundle:decode_table("LevelExp", LevelExp.decode)),
        _character = CharacterTable.decode(bundle:decode_table("Character", Character.decode)),
        _character_skill = CharacterSkillTable.decode(bundle:decode_table("CharacterSkill", CharacterSkill.decode)),
        _buff = BuffTable.decode(bundle:decode_table("Buff", Buff.decode)),
        _drop_group = DropGroupTable.decode(bundle:decode_table("DropGroup", DropGroup.decode)),
        _drop_entry = DropEntryTable.decode(bundle:decode_table("DropEntry", DropEntry.decode)),
        _monster = MonsterTable.decode(bundle:decode_table("Monster", Monster.decode)),
        _stage = StageTable.decode(bundle:decode_table("Stage", Stage.decode)),
        _stage_reward = StageRewardTable.decode(bundle:decode_table("StageReward", StageReward.decode)),
        _dungeon = DungeonTable.decode(bundle:decode_table("Dungeon", Dungeon.decode)),
        _shop = ShopTable.decode(bundle:decode_table("Shop", Shop.decode)),
        _shop_item = ShopItemTable.decode(bundle:decode_table("ShopItem", ShopItem.decode)),
        _recipe = RecipeTable.decode(bundle:decode_table("Recipe", Recipe.decode)),
        _gacha_pool = GachaPoolTable.decode(bundle:decode_table("GachaPool", GachaPool.decode)),
        _gacha_item = GachaItemTable.decode(bundle:decode_table("GachaItem", GachaItem.decode)),
        _equipment_set = EquipmentSetTable.decode(bundle:decode_table("EquipmentSet", EquipmentSet.decode)),
        _achievement = AchievementTable.decode(bundle:decode_table("Achievement", Achievement.decode)),
        _vip_level = VipLevelTable.decode(bundle:decode_table("VipLevel", VipLevel.decode)),
        _mail_template = MailTemplateTable.decode(bundle:decode_table("MailTemplate", MailTemplate.decode)),
        _mail_reward = MailRewardTable.decode(bundle:decode_table("MailReward", MailReward.decode)),
        _dialogue = DialogueTable.decode(bundle:decode_table("Dialogue", Dialogue.decode)),
        _event_rule = EventRuleTable.decode(bundle:decode_table("EventRule", EventRule.decode)),
    }, SoraConfig)
end

---@return table[]
function SoraConfig:tables()
    return {
        self._item,
        self._skill,
        self._quest,
        self._quest_reward,
        self._game_settings,
        self._localization,
        self._level_exp,
        self._character,
        self._character_skill,
        self._buff,
        self._drop_group,
        self._drop_entry,
        self._monster,
        self._stage,
        self._stage_reward,
        self._dungeon,
        self._shop,
        self._shop_item,
        self._recipe,
        self._gacha_pool,
        self._gacha_item,
        self._equipment_set,
        self._achievement,
        self._vip_level,
        self._mail_template,
        self._mail_reward,
        self._dialogue,
        self._event_rule,
    }
end
---@return ItemTable
function SoraConfig:item()
    return self._item
end
---@return SkillTable
function SoraConfig:skill()
    return self._skill
end
---@return QuestTable
function SoraConfig:quest()
    return self._quest
end
---@return QuestRewardTable
function SoraConfig:quest_reward()
    return self._quest_reward
end
---@return GameSettingsTable
function SoraConfig:game_settings()
    return self._game_settings
end
---@return LocalizationTable
function SoraConfig:localization()
    return self._localization
end
---@return LevelExpTable
function SoraConfig:level_exp()
    return self._level_exp
end
---@return CharacterTable
function SoraConfig:character()
    return self._character
end
---@return CharacterSkillTable
function SoraConfig:character_skill()
    return self._character_skill
end
---@return BuffTable
function SoraConfig:buff()
    return self._buff
end
---@return DropGroupTable
function SoraConfig:drop_group()
    return self._drop_group
end
---@return DropEntryTable
function SoraConfig:drop_entry()
    return self._drop_entry
end
---@return MonsterTable
function SoraConfig:monster()
    return self._monster
end
---@return StageTable
function SoraConfig:stage()
    return self._stage
end
---@return StageRewardTable
function SoraConfig:stage_reward()
    return self._stage_reward
end
---@return DungeonTable
function SoraConfig:dungeon()
    return self._dungeon
end
---@return ShopTable
function SoraConfig:shop()
    return self._shop
end
---@return ShopItemTable
function SoraConfig:shop_item()
    return self._shop_item
end
---@return RecipeTable
function SoraConfig:recipe()
    return self._recipe
end
---@return GachaPoolTable
function SoraConfig:gacha_pool()
    return self._gacha_pool
end
---@return GachaItemTable
function SoraConfig:gacha_item()
    return self._gacha_item
end
---@return EquipmentSetTable
function SoraConfig:equipment_set()
    return self._equipment_set
end
---@return AchievementTable
function SoraConfig:achievement()
    return self._achievement
end
---@return VipLevelTable
function SoraConfig:vip_level()
    return self._vip_level
end
---@return MailTemplateTable
function SoraConfig:mail_template()
    return self._mail_template
end
---@return MailRewardTable
function SoraConfig:mail_reward()
    return self._mail_reward
end
---@return DialogueTable
function SoraConfig:dialogue()
    return self._dialogue
end
---@return EventRuleTable
function SoraConfig:event_rule()
    return self._event_rule
end
return SoraConfig
