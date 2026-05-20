package showcase

import "fmt"

type SoraTableMode int

const (
	SoraTableModeList SoraTableMode = iota
	SoraTableModeMap
	SoraTableModeSingleton
)

type SoraTable interface {
	Name() string
	Mode() SoraTableMode
	Key() string
	RowType() string
	Len() int
}
type ItemTable struct {
	rows   map[int32]Item
	byName map[string]Item
}

func decodeItemTable(bundle *SoraBundle) (*ItemTable, error) {
	rows, err := DecodeTable(bundle, "Item", decodeItem)
	if err != nil {
		return nil, err
	}
	return &ItemTable{
		rows:   DecodeMapTable(rows, func(row Item) int32 { return row.Id }),
		byName: DecodeUniqueIndex(rows, func(row Item) string { return row.Name }),
	}, nil
}

func (table *ItemTable) Rows() map[int32]Item {
	return table.rows
}
func (table *ItemTable) Get(key int32) (Item, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *ItemTable) GetByName(name string) (Item, bool) {
	value, ok := table.byName[name]
	return value, ok
}
func (table *ItemTable) Name() string {
	return "Item"
}

func (table *ItemTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *ItemTable) Key() string {
	return "id"
}

func (table *ItemTable) RowType() string {
	return "Item"
}

func (table *ItemTable) Len() int {
	return len(table.rows)
}

type SkillTable struct {
	rows map[int32]Skill
}

func decodeSkillTable(bundle *SoraBundle) (*SkillTable, error) {
	rows, err := DecodeTable(bundle, "Skill", decodeSkill)
	if err != nil {
		return nil, err
	}
	return &SkillTable{rows: DecodeMapTable(rows, func(row Skill) int32 { return row.Id })}, nil
}

func (table *SkillTable) Rows() map[int32]Skill {
	return table.rows
}
func (table *SkillTable) Get(key int32) (Skill, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *SkillTable) Name() string {
	return "Skill"
}

func (table *SkillTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *SkillTable) Key() string {
	return "id"
}

func (table *SkillTable) RowType() string {
	return "Skill"
}

func (table *SkillTable) Len() int {
	return len(table.rows)
}

type QuestTable struct {
	rows map[int32]Quest
}

func decodeQuestTable(bundle *SoraBundle) (*QuestTable, error) {
	rows, err := DecodeTable(bundle, "Quest", decodeQuest)
	if err != nil {
		return nil, err
	}
	return &QuestTable{rows: DecodeMapTable(rows, func(row Quest) int32 { return row.Id })}, nil
}

func (table *QuestTable) Rows() map[int32]Quest {
	return table.rows
}
func (table *QuestTable) Get(key int32) (Quest, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *QuestTable) Name() string {
	return "Quest"
}

func (table *QuestTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *QuestTable) Key() string {
	return "id"
}

func (table *QuestTable) RowType() string {
	return "Quest"
}

func (table *QuestTable) Len() int {
	return len(table.rows)
}

type QuestRewardTable struct {
	rows []QuestReward
}

func decodeQuestRewardTable(bundle *SoraBundle) (*QuestRewardTable, error) {
	rows, err := DecodeTable(bundle, "QuestReward", decodeQuestReward)
	if err != nil {
		return nil, err
	}
	return &QuestRewardTable{rows: rows}, nil
}

func (table *QuestRewardTable) Rows() []QuestReward {
	return table.rows
}
func (table *QuestRewardTable) Name() string {
	return "QuestReward"
}

func (table *QuestRewardTable) Mode() SoraTableMode {
	return SoraTableModeList
}

func (table *QuestRewardTable) Key() string {
	return ""
}

func (table *QuestRewardTable) RowType() string {
	return "QuestReward"
}

func (table *QuestRewardTable) Len() int {
	return len(table.rows)
}

type GameSettingsTable struct {
	rows GameSettings
}

func decodeGameSettingsTable(bundle *SoraBundle) (*GameSettingsTable, error) {
	rows, err := DecodeTable(bundle, "GameSettings", decodeGameSettings)
	if err != nil {
		return nil, err
	}
	row, err := RequireSingletonTable(rows, "GameSettings")
	if err != nil {
		return nil, err
	}
	return &GameSettingsTable{rows: row}, nil
}

func (table *GameSettingsTable) Rows() GameSettings {
	return table.rows
}
func (table *GameSettingsTable) Name() string {
	return "GameSettings"
}

func (table *GameSettingsTable) Mode() SoraTableMode {
	return SoraTableModeSingleton
}

func (table *GameSettingsTable) Key() string {
	return ""
}

func (table *GameSettingsTable) RowType() string {
	return "GameSettings"
}

func (table *GameSettingsTable) Len() int {
	return 1
}

type LocalizationTable struct {
	rows map[string]Localization
}

func decodeLocalizationTable(bundle *SoraBundle) (*LocalizationTable, error) {
	rows, err := DecodeTable(bundle, "Localization", decodeLocalization)
	if err != nil {
		return nil, err
	}
	return &LocalizationTable{rows: DecodeMapTable(rows, func(row Localization) string { return row.Key })}, nil
}

func (table *LocalizationTable) Rows() map[string]Localization {
	return table.rows
}
func (table *LocalizationTable) Get(key string) (Localization, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *LocalizationTable) Name() string {
	return "Localization"
}

func (table *LocalizationTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *LocalizationTable) Key() string {
	return "key"
}

func (table *LocalizationTable) RowType() string {
	return "Localization"
}

func (table *LocalizationTable) Len() int {
	return len(table.rows)
}

type LevelExpTable struct {
	rows map[int32]LevelExp
}

func decodeLevelExpTable(bundle *SoraBundle) (*LevelExpTable, error) {
	rows, err := DecodeTable(bundle, "LevelExp", decodeLevelExp)
	if err != nil {
		return nil, err
	}
	return &LevelExpTable{rows: DecodeMapTable(rows, func(row LevelExp) int32 { return row.Level })}, nil
}

func (table *LevelExpTable) Rows() map[int32]LevelExp {
	return table.rows
}
func (table *LevelExpTable) Get(key int32) (LevelExp, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *LevelExpTable) Name() string {
	return "LevelExp"
}

func (table *LevelExpTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *LevelExpTable) Key() string {
	return "level"
}

func (table *LevelExpTable) RowType() string {
	return "LevelExp"
}

func (table *LevelExpTable) Len() int {
	return len(table.rows)
}

type CharacterTable struct {
	rows map[int32]Character
}

func decodeCharacterTable(bundle *SoraBundle) (*CharacterTable, error) {
	rows, err := DecodeTable(bundle, "Character", decodeCharacter)
	if err != nil {
		return nil, err
	}
	return &CharacterTable{rows: DecodeMapTable(rows, func(row Character) int32 { return row.Id })}, nil
}

func (table *CharacterTable) Rows() map[int32]Character {
	return table.rows
}
func (table *CharacterTable) Get(key int32) (Character, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *CharacterTable) Name() string {
	return "Character"
}

func (table *CharacterTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *CharacterTable) Key() string {
	return "id"
}

func (table *CharacterTable) RowType() string {
	return "Character"
}

func (table *CharacterTable) Len() int {
	return len(table.rows)
}

type CharacterSkillTable struct {
	rows []CharacterSkill
}

func decodeCharacterSkillTable(bundle *SoraBundle) (*CharacterSkillTable, error) {
	rows, err := DecodeTable(bundle, "CharacterSkill", decodeCharacterSkill)
	if err != nil {
		return nil, err
	}
	return &CharacterSkillTable{rows: rows}, nil
}

func (table *CharacterSkillTable) Rows() []CharacterSkill {
	return table.rows
}
func (table *CharacterSkillTable) Name() string {
	return "CharacterSkill"
}

func (table *CharacterSkillTable) Mode() SoraTableMode {
	return SoraTableModeList
}

func (table *CharacterSkillTable) Key() string {
	return ""
}

func (table *CharacterSkillTable) RowType() string {
	return "CharacterSkill"
}

func (table *CharacterSkillTable) Len() int {
	return len(table.rows)
}

type BuffTable struct {
	rows map[int32]Buff
}

func decodeBuffTable(bundle *SoraBundle) (*BuffTable, error) {
	rows, err := DecodeTable(bundle, "Buff", decodeBuff)
	if err != nil {
		return nil, err
	}
	return &BuffTable{rows: DecodeMapTable(rows, func(row Buff) int32 { return row.Id })}, nil
}

func (table *BuffTable) Rows() map[int32]Buff {
	return table.rows
}
func (table *BuffTable) Get(key int32) (Buff, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *BuffTable) Name() string {
	return "Buff"
}

func (table *BuffTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *BuffTable) Key() string {
	return "id"
}

func (table *BuffTable) RowType() string {
	return "Buff"
}

func (table *BuffTable) Len() int {
	return len(table.rows)
}

type DropGroupTable struct {
	rows map[int32]DropGroup
}

func decodeDropGroupTable(bundle *SoraBundle) (*DropGroupTable, error) {
	rows, err := DecodeTable(bundle, "DropGroup", decodeDropGroup)
	if err != nil {
		return nil, err
	}
	return &DropGroupTable{rows: DecodeMapTable(rows, func(row DropGroup) int32 { return row.Id })}, nil
}

func (table *DropGroupTable) Rows() map[int32]DropGroup {
	return table.rows
}
func (table *DropGroupTable) Get(key int32) (DropGroup, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *DropGroupTable) Name() string {
	return "DropGroup"
}

func (table *DropGroupTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *DropGroupTable) Key() string {
	return "id"
}

func (table *DropGroupTable) RowType() string {
	return "DropGroup"
}

func (table *DropGroupTable) Len() int {
	return len(table.rows)
}

type DropEntryTable struct {
	rows []DropEntry
}

func decodeDropEntryTable(bundle *SoraBundle) (*DropEntryTable, error) {
	rows, err := DecodeTable(bundle, "DropEntry", decodeDropEntry)
	if err != nil {
		return nil, err
	}
	return &DropEntryTable{rows: rows}, nil
}

func (table *DropEntryTable) Rows() []DropEntry {
	return table.rows
}
func (table *DropEntryTable) Name() string {
	return "DropEntry"
}

func (table *DropEntryTable) Mode() SoraTableMode {
	return SoraTableModeList
}

func (table *DropEntryTable) Key() string {
	return ""
}

func (table *DropEntryTable) RowType() string {
	return "DropEntry"
}

func (table *DropEntryTable) Len() int {
	return len(table.rows)
}

type MonsterTable struct {
	rows map[int32]Monster
}

func decodeMonsterTable(bundle *SoraBundle) (*MonsterTable, error) {
	rows, err := DecodeTable(bundle, "Monster", decodeMonster)
	if err != nil {
		return nil, err
	}
	return &MonsterTable{rows: DecodeMapTable(rows, func(row Monster) int32 { return row.Id })}, nil
}

func (table *MonsterTable) Rows() map[int32]Monster {
	return table.rows
}
func (table *MonsterTable) Get(key int32) (Monster, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *MonsterTable) Name() string {
	return "Monster"
}

func (table *MonsterTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *MonsterTable) Key() string {
	return "id"
}

func (table *MonsterTable) RowType() string {
	return "Monster"
}

func (table *MonsterTable) Len() int {
	return len(table.rows)
}

type StageTable struct {
	rows map[int32]Stage
}

func decodeStageTable(bundle *SoraBundle) (*StageTable, error) {
	rows, err := DecodeTable(bundle, "Stage", decodeStage)
	if err != nil {
		return nil, err
	}
	return &StageTable{rows: DecodeMapTable(rows, func(row Stage) int32 { return row.Id })}, nil
}

func (table *StageTable) Rows() map[int32]Stage {
	return table.rows
}
func (table *StageTable) Get(key int32) (Stage, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *StageTable) Name() string {
	return "Stage"
}

func (table *StageTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *StageTable) Key() string {
	return "id"
}

func (table *StageTable) RowType() string {
	return "Stage"
}

func (table *StageTable) Len() int {
	return len(table.rows)
}

type StageRewardTable struct {
	rows []StageReward
}

func decodeStageRewardTable(bundle *SoraBundle) (*StageRewardTable, error) {
	rows, err := DecodeTable(bundle, "StageReward", decodeStageReward)
	if err != nil {
		return nil, err
	}
	return &StageRewardTable{rows: rows}, nil
}

func (table *StageRewardTable) Rows() []StageReward {
	return table.rows
}
func (table *StageRewardTable) Name() string {
	return "StageReward"
}

func (table *StageRewardTable) Mode() SoraTableMode {
	return SoraTableModeList
}

func (table *StageRewardTable) Key() string {
	return ""
}

func (table *StageRewardTable) RowType() string {
	return "StageReward"
}

func (table *StageRewardTable) Len() int {
	return len(table.rows)
}

type DungeonTable struct {
	rows map[int32]Dungeon
}

func decodeDungeonTable(bundle *SoraBundle) (*DungeonTable, error) {
	rows, err := DecodeTable(bundle, "Dungeon", decodeDungeon)
	if err != nil {
		return nil, err
	}
	return &DungeonTable{rows: DecodeMapTable(rows, func(row Dungeon) int32 { return row.Id })}, nil
}

func (table *DungeonTable) Rows() map[int32]Dungeon {
	return table.rows
}
func (table *DungeonTable) Get(key int32) (Dungeon, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *DungeonTable) Name() string {
	return "Dungeon"
}

func (table *DungeonTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *DungeonTable) Key() string {
	return "id"
}

func (table *DungeonTable) RowType() string {
	return "Dungeon"
}

func (table *DungeonTable) Len() int {
	return len(table.rows)
}

type ShopTable struct {
	rows map[int32]Shop
}

func decodeShopTable(bundle *SoraBundle) (*ShopTable, error) {
	rows, err := DecodeTable(bundle, "Shop", decodeShop)
	if err != nil {
		return nil, err
	}
	return &ShopTable{rows: DecodeMapTable(rows, func(row Shop) int32 { return row.Id })}, nil
}

func (table *ShopTable) Rows() map[int32]Shop {
	return table.rows
}
func (table *ShopTable) Get(key int32) (Shop, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *ShopTable) Name() string {
	return "Shop"
}

func (table *ShopTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *ShopTable) Key() string {
	return "id"
}

func (table *ShopTable) RowType() string {
	return "Shop"
}

func (table *ShopTable) Len() int {
	return len(table.rows)
}

type ShopItemTable struct {
	rows []ShopItem
}

func decodeShopItemTable(bundle *SoraBundle) (*ShopItemTable, error) {
	rows, err := DecodeTable(bundle, "ShopItem", decodeShopItem)
	if err != nil {
		return nil, err
	}
	return &ShopItemTable{rows: rows}, nil
}

func (table *ShopItemTable) Rows() []ShopItem {
	return table.rows
}
func (table *ShopItemTable) Name() string {
	return "ShopItem"
}

func (table *ShopItemTable) Mode() SoraTableMode {
	return SoraTableModeList
}

func (table *ShopItemTable) Key() string {
	return ""
}

func (table *ShopItemTable) RowType() string {
	return "ShopItem"
}

func (table *ShopItemTable) Len() int {
	return len(table.rows)
}

type RecipeTable struct {
	rows map[int32]Recipe
}

func decodeRecipeTable(bundle *SoraBundle) (*RecipeTable, error) {
	rows, err := DecodeTable(bundle, "Recipe", decodeRecipe)
	if err != nil {
		return nil, err
	}
	return &RecipeTable{rows: DecodeMapTable(rows, func(row Recipe) int32 { return row.Id })}, nil
}

func (table *RecipeTable) Rows() map[int32]Recipe {
	return table.rows
}
func (table *RecipeTable) Get(key int32) (Recipe, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *RecipeTable) Name() string {
	return "Recipe"
}

func (table *RecipeTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *RecipeTable) Key() string {
	return "id"
}

func (table *RecipeTable) RowType() string {
	return "Recipe"
}

func (table *RecipeTable) Len() int {
	return len(table.rows)
}

type GachaPoolTable struct {
	rows map[int32]GachaPool
}

func decodeGachaPoolTable(bundle *SoraBundle) (*GachaPoolTable, error) {
	rows, err := DecodeTable(bundle, "GachaPool", decodeGachaPool)
	if err != nil {
		return nil, err
	}
	return &GachaPoolTable{rows: DecodeMapTable(rows, func(row GachaPool) int32 { return row.Id })}, nil
}

func (table *GachaPoolTable) Rows() map[int32]GachaPool {
	return table.rows
}
func (table *GachaPoolTable) Get(key int32) (GachaPool, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *GachaPoolTable) Name() string {
	return "GachaPool"
}

func (table *GachaPoolTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *GachaPoolTable) Key() string {
	return "id"
}

func (table *GachaPoolTable) RowType() string {
	return "GachaPool"
}

func (table *GachaPoolTable) Len() int {
	return len(table.rows)
}

type GachaItemTable struct {
	rows []GachaItem
}

func decodeGachaItemTable(bundle *SoraBundle) (*GachaItemTable, error) {
	rows, err := DecodeTable(bundle, "GachaItem", decodeGachaItem)
	if err != nil {
		return nil, err
	}
	return &GachaItemTable{rows: rows}, nil
}

func (table *GachaItemTable) Rows() []GachaItem {
	return table.rows
}
func (table *GachaItemTable) Name() string {
	return "GachaItem"
}

func (table *GachaItemTable) Mode() SoraTableMode {
	return SoraTableModeList
}

func (table *GachaItemTable) Key() string {
	return ""
}

func (table *GachaItemTable) RowType() string {
	return "GachaItem"
}

func (table *GachaItemTable) Len() int {
	return len(table.rows)
}

type EquipmentSetTable struct {
	rows map[int32]EquipmentSet
}

func decodeEquipmentSetTable(bundle *SoraBundle) (*EquipmentSetTable, error) {
	rows, err := DecodeTable(bundle, "EquipmentSet", decodeEquipmentSet)
	if err != nil {
		return nil, err
	}
	return &EquipmentSetTable{rows: DecodeMapTable(rows, func(row EquipmentSet) int32 { return row.Id })}, nil
}

func (table *EquipmentSetTable) Rows() map[int32]EquipmentSet {
	return table.rows
}
func (table *EquipmentSetTable) Get(key int32) (EquipmentSet, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *EquipmentSetTable) Name() string {
	return "EquipmentSet"
}

func (table *EquipmentSetTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *EquipmentSetTable) Key() string {
	return "id"
}

func (table *EquipmentSetTable) RowType() string {
	return "EquipmentSet"
}

func (table *EquipmentSetTable) Len() int {
	return len(table.rows)
}

type AchievementTable struct {
	rows map[int32]Achievement
}

func decodeAchievementTable(bundle *SoraBundle) (*AchievementTable, error) {
	rows, err := DecodeTable(bundle, "Achievement", decodeAchievement)
	if err != nil {
		return nil, err
	}
	return &AchievementTable{rows: DecodeMapTable(rows, func(row Achievement) int32 { return row.Id })}, nil
}

func (table *AchievementTable) Rows() map[int32]Achievement {
	return table.rows
}
func (table *AchievementTable) Get(key int32) (Achievement, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *AchievementTable) Name() string {
	return "Achievement"
}

func (table *AchievementTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *AchievementTable) Key() string {
	return "id"
}

func (table *AchievementTable) RowType() string {
	return "Achievement"
}

func (table *AchievementTable) Len() int {
	return len(table.rows)
}

type VipLevelTable struct {
	rows map[int32]VipLevel
}

func decodeVipLevelTable(bundle *SoraBundle) (*VipLevelTable, error) {
	rows, err := DecodeTable(bundle, "VipLevel", decodeVipLevel)
	if err != nil {
		return nil, err
	}
	return &VipLevelTable{rows: DecodeMapTable(rows, func(row VipLevel) int32 { return row.Level })}, nil
}

func (table *VipLevelTable) Rows() map[int32]VipLevel {
	return table.rows
}
func (table *VipLevelTable) Get(key int32) (VipLevel, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *VipLevelTable) Name() string {
	return "VipLevel"
}

func (table *VipLevelTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *VipLevelTable) Key() string {
	return "level"
}

func (table *VipLevelTable) RowType() string {
	return "VipLevel"
}

func (table *VipLevelTable) Len() int {
	return len(table.rows)
}

type MailTemplateTable struct {
	rows map[int32]MailTemplate
}

func decodeMailTemplateTable(bundle *SoraBundle) (*MailTemplateTable, error) {
	rows, err := DecodeTable(bundle, "MailTemplate", decodeMailTemplate)
	if err != nil {
		return nil, err
	}
	return &MailTemplateTable{rows: DecodeMapTable(rows, func(row MailTemplate) int32 { return row.Id })}, nil
}

func (table *MailTemplateTable) Rows() map[int32]MailTemplate {
	return table.rows
}
func (table *MailTemplateTable) Get(key int32) (MailTemplate, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *MailTemplateTable) Name() string {
	return "MailTemplate"
}

func (table *MailTemplateTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *MailTemplateTable) Key() string {
	return "id"
}

func (table *MailTemplateTable) RowType() string {
	return "MailTemplate"
}

func (table *MailTemplateTable) Len() int {
	return len(table.rows)
}

type MailRewardTable struct {
	rows []MailReward
}

func decodeMailRewardTable(bundle *SoraBundle) (*MailRewardTable, error) {
	rows, err := DecodeTable(bundle, "MailReward", decodeMailReward)
	if err != nil {
		return nil, err
	}
	return &MailRewardTable{rows: rows}, nil
}

func (table *MailRewardTable) Rows() []MailReward {
	return table.rows
}
func (table *MailRewardTable) Name() string {
	return "MailReward"
}

func (table *MailRewardTable) Mode() SoraTableMode {
	return SoraTableModeList
}

func (table *MailRewardTable) Key() string {
	return ""
}

func (table *MailRewardTable) RowType() string {
	return "MailReward"
}

func (table *MailRewardTable) Len() int {
	return len(table.rows)
}

type DialogueTable struct {
	rows map[int32]Dialogue
}

func decodeDialogueTable(bundle *SoraBundle) (*DialogueTable, error) {
	rows, err := DecodeTable(bundle, "Dialogue", decodeDialogue)
	if err != nil {
		return nil, err
	}
	return &DialogueTable{rows: DecodeMapTable(rows, func(row Dialogue) int32 { return row.Id })}, nil
}

func (table *DialogueTable) Rows() map[int32]Dialogue {
	return table.rows
}
func (table *DialogueTable) Get(key int32) (Dialogue, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *DialogueTable) Name() string {
	return "Dialogue"
}

func (table *DialogueTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *DialogueTable) Key() string {
	return "id"
}

func (table *DialogueTable) RowType() string {
	return "Dialogue"
}

func (table *DialogueTable) Len() int {
	return len(table.rows)
}

type EventRuleTable struct {
	rows map[int32]EventRule
}

func decodeEventRuleTable(bundle *SoraBundle) (*EventRuleTable, error) {
	rows, err := DecodeTable(bundle, "EventRule", decodeEventRule)
	if err != nil {
		return nil, err
	}
	return &EventRuleTable{rows: DecodeMapTable(rows, func(row EventRule) int32 { return row.Id })}, nil
}

func (table *EventRuleTable) Rows() map[int32]EventRule {
	return table.rows
}
func (table *EventRuleTable) Get(key int32) (EventRule, bool) {
	value, ok := table.rows[key]
	return value, ok
}
func (table *EventRuleTable) Name() string {
	return "EventRule"
}

func (table *EventRuleTable) Mode() SoraTableMode {
	return SoraTableModeMap
}

func (table *EventRuleTable) Key() string {
	return "id"
}

func (table *EventRuleTable) RowType() string {
	return "EventRule"
}

func (table *EventRuleTable) Len() int {
	return len(table.rows)
}

type SoraConfig struct {
	tables map[string]SoraTable
}

func NewSoraConfigFromBytes(bytes []byte) (*SoraConfig, error) {
	bundle, err := ParseSoraBundle(bytes)
	if err != nil {
		return nil, err
	}
	tables := make(map[string]SoraTable, 28)
	itemTable, err := decodeItemTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Item"] = itemTable
	skillTable, err := decodeSkillTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Skill"] = skillTable
	questTable, err := decodeQuestTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Quest"] = questTable
	questRewardTable, err := decodeQuestRewardTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["QuestReward"] = questRewardTable
	gameSettingsTable, err := decodeGameSettingsTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["GameSettings"] = gameSettingsTable
	localizationTable, err := decodeLocalizationTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Localization"] = localizationTable
	levelExpTable, err := decodeLevelExpTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["LevelExp"] = levelExpTable
	characterTable, err := decodeCharacterTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Character"] = characterTable
	characterSkillTable, err := decodeCharacterSkillTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["CharacterSkill"] = characterSkillTable
	buffTable, err := decodeBuffTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Buff"] = buffTable
	dropGroupTable, err := decodeDropGroupTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["DropGroup"] = dropGroupTable
	dropEntryTable, err := decodeDropEntryTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["DropEntry"] = dropEntryTable
	monsterTable, err := decodeMonsterTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Monster"] = monsterTable
	stageTable, err := decodeStageTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Stage"] = stageTable
	stageRewardTable, err := decodeStageRewardTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["StageReward"] = stageRewardTable
	dungeonTable, err := decodeDungeonTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Dungeon"] = dungeonTable
	shopTable, err := decodeShopTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Shop"] = shopTable
	shopItemTable, err := decodeShopItemTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["ShopItem"] = shopItemTable
	recipeTable, err := decodeRecipeTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Recipe"] = recipeTable
	gachaPoolTable, err := decodeGachaPoolTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["GachaPool"] = gachaPoolTable
	gachaItemTable, err := decodeGachaItemTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["GachaItem"] = gachaItemTable
	equipmentSetTable, err := decodeEquipmentSetTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["EquipmentSet"] = equipmentSetTable
	achievementTable, err := decodeAchievementTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Achievement"] = achievementTable
	vipLevelTable, err := decodeVipLevelTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["VipLevel"] = vipLevelTable
	mailTemplateTable, err := decodeMailTemplateTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["MailTemplate"] = mailTemplateTable
	mailRewardTable, err := decodeMailRewardTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["MailReward"] = mailRewardTable
	dialogueTable, err := decodeDialogueTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["Dialogue"] = dialogueTable
	eventRuleTable, err := decodeEventRuleTable(bundle)
	if err != nil {
		return nil, err
	}
	tables["EventRule"] = eventRuleTable
	return &SoraConfig{tables: tables}, nil
}

func (config *SoraConfig) Tables() []SoraTable {
	tables := make([]SoraTable, 0, len(config.tables))
	for _, table := range config.tables {
		tables = append(tables, table)
	}
	return tables
}
func (config *SoraConfig) Item() *ItemTable {
	return config.tables["Item"].(*ItemTable)
}
func (config *SoraConfig) Skill() *SkillTable {
	return config.tables["Skill"].(*SkillTable)
}
func (config *SoraConfig) Quest() *QuestTable {
	return config.tables["Quest"].(*QuestTable)
}
func (config *SoraConfig) QuestReward() *QuestRewardTable {
	return config.tables["QuestReward"].(*QuestRewardTable)
}
func (config *SoraConfig) GameSettings() *GameSettingsTable {
	return config.tables["GameSettings"].(*GameSettingsTable)
}
func (config *SoraConfig) Localization() *LocalizationTable {
	return config.tables["Localization"].(*LocalizationTable)
}
func (config *SoraConfig) LevelExp() *LevelExpTable {
	return config.tables["LevelExp"].(*LevelExpTable)
}
func (config *SoraConfig) Character() *CharacterTable {
	return config.tables["Character"].(*CharacterTable)
}
func (config *SoraConfig) CharacterSkill() *CharacterSkillTable {
	return config.tables["CharacterSkill"].(*CharacterSkillTable)
}
func (config *SoraConfig) Buff() *BuffTable {
	return config.tables["Buff"].(*BuffTable)
}
func (config *SoraConfig) DropGroup() *DropGroupTable {
	return config.tables["DropGroup"].(*DropGroupTable)
}
func (config *SoraConfig) DropEntry() *DropEntryTable {
	return config.tables["DropEntry"].(*DropEntryTable)
}
func (config *SoraConfig) Monster() *MonsterTable {
	return config.tables["Monster"].(*MonsterTable)
}
func (config *SoraConfig) Stage() *StageTable {
	return config.tables["Stage"].(*StageTable)
}
func (config *SoraConfig) StageReward() *StageRewardTable {
	return config.tables["StageReward"].(*StageRewardTable)
}
func (config *SoraConfig) Dungeon() *DungeonTable {
	return config.tables["Dungeon"].(*DungeonTable)
}
func (config *SoraConfig) Shop() *ShopTable {
	return config.tables["Shop"].(*ShopTable)
}
func (config *SoraConfig) ShopItem() *ShopItemTable {
	return config.tables["ShopItem"].(*ShopItemTable)
}
func (config *SoraConfig) Recipe() *RecipeTable {
	return config.tables["Recipe"].(*RecipeTable)
}
func (config *SoraConfig) GachaPool() *GachaPoolTable {
	return config.tables["GachaPool"].(*GachaPoolTable)
}
func (config *SoraConfig) GachaItem() *GachaItemTable {
	return config.tables["GachaItem"].(*GachaItemTable)
}
func (config *SoraConfig) EquipmentSet() *EquipmentSetTable {
	return config.tables["EquipmentSet"].(*EquipmentSetTable)
}
func (config *SoraConfig) Achievement() *AchievementTable {
	return config.tables["Achievement"].(*AchievementTable)
}
func (config *SoraConfig) VipLevel() *VipLevelTable {
	return config.tables["VipLevel"].(*VipLevelTable)
}
func (config *SoraConfig) MailTemplate() *MailTemplateTable {
	return config.tables["MailTemplate"].(*MailTemplateTable)
}
func (config *SoraConfig) MailReward() *MailRewardTable {
	return config.tables["MailReward"].(*MailRewardTable)
}
func (config *SoraConfig) Dialogue() *DialogueTable {
	return config.tables["Dialogue"].(*DialogueTable)
}
func (config *SoraConfig) EventRule() *EventRuleTable {
	return config.tables["EventRule"].(*EventRuleTable)
}
func DecodeMapTable[K comparable, V any](rows []V, key func(V) K) map[K]V {
	values := make(map[K]V, len(rows))
	for _, row := range rows {
		values[key(row)] = row
	}
	return values
}

func DecodeUniqueIndex[K comparable, V any](rows []V, key func(V) K) map[K]V {
	values := make(map[K]V, len(rows))
	for _, row := range rows {
		values[key(row)] = row
	}
	return values
}

func RequireSingletonTable[T any](rows []T, name string) (T, error) {
	var zero T
	if len(rows) != 1 {
		return zero, fmt.Errorf("expected singleton table `%s` to contain exactly 1 row, got %d", name, len(rows))
	}
	return rows[0], nil
}
