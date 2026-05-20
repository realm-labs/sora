#![allow(dead_code)]

pub mod achievement;
pub mod buff;
pub mod character;
pub mod character_skill;
pub mod dialogue;
pub mod drop_entry;
pub mod drop_group;
pub mod dungeon;
pub mod element_type;
pub mod equipment_set;
pub mod event_condition;
pub mod event_rule;
pub mod gacha_item;
pub mod gacha_pool;
pub mod game_settings;
pub mod item;
pub mod item_type;
pub mod level_exp;
pub mod localization;
pub mod mail_reward;
pub mod mail_template;
pub mod mail_type;
pub mod monster;
pub mod quest;
pub mod quest_reward;
pub mod quest_type;
pub mod rarity;
pub mod recipe;
pub mod resource_cost;
pub mod resource_kind;
pub mod reward;
pub mod reward_action;
pub mod runtime;
pub mod shop;
pub mod shop_item;
pub mod skill;
pub mod skill_effect;
pub mod stage;
pub mod stage_reward;
pub mod stat_modifier;
pub mod stat_type;
pub mod vec3;
pub mod vip_level;
pub type SoraMap<K, V> = rustc_hash::FxHashMap<K, V>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoraTableMode {
    List,
    Map,
    Singleton,
}

pub trait SoraTable: std::any::Any + Send + Sync {
    fn name(&self) -> &'static str;
    fn mode(&self) -> SoraTableMode;
    fn key(&self) -> Option<&'static str>;
    fn row_type(&self) -> &'static str;
    fn len(&self) -> usize;
}

pub struct SoraConfig {
    tables: SoraMap<&'static str, Box<dyn SoraTable>>,
}

impl std::fmt::Debug for SoraConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tables = self.tables.keys().copied().collect::<Vec<_>>();
        tables.sort_unstable();
        f.debug_struct("SoraConfig")
            .field("tables", &tables)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ItemTable {
    rows: SoraMap<i32, item::Item>,
    by_name: SoraMap<String, i32>,
}

impl ItemTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        let rows = bundle.decode_table::<item::Item>("Item")?;
        let by_name = build_unique_map_index(rows.iter(), |row| row.name.clone(), |row| row.id);
        let rows = decode_map_table(rows, |row| row.id);
        Ok(Self { rows, by_name })
    }
    pub fn get(&self, key: i32) -> Option<&item::Item> {
        self.rows.get(&key)
    }
    pub fn get_by_name(&self, name: &str) -> Option<&item::Item> {
        self.by_name.get(name).and_then(|key| self.rows.get(key))
    }
}

impl std::ops::Deref for ItemTable {
    type Target = SoraMap<i32, item::Item>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for ItemTable {
    fn name(&self) -> &'static str {
        "Item"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "item::Item"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct SkillTable {
    rows: SoraMap<i32, skill::Skill>,
}

impl SkillTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(bundle.decode_table::<skill::Skill>("Skill")?, |row| row.id),
        })
    }
    pub fn get(&self, key: i32) -> Option<&skill::Skill> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for SkillTable {
    type Target = SoraMap<i32, skill::Skill>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for SkillTable {
    fn name(&self) -> &'static str {
        "Skill"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "skill::Skill"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct QuestTable {
    rows: SoraMap<i32, quest::Quest>,
}

impl QuestTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(bundle.decode_table::<quest::Quest>("Quest")?, |row| row.id),
        })
    }
    pub fn get(&self, key: i32) -> Option<&quest::Quest> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for QuestTable {
    type Target = SoraMap<i32, quest::Quest>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for QuestTable {
    fn name(&self) -> &'static str {
        "Quest"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "quest::Quest"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct QuestRewardTable {
    rows: Vec<quest_reward::QuestReward>,
}

impl QuestRewardTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: bundle.decode_table::<quest_reward::QuestReward>("QuestReward")?,
        })
    }
}

impl std::ops::Deref for QuestRewardTable {
    type Target = Vec<quest_reward::QuestReward>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for QuestRewardTable {
    fn name(&self) -> &'static str {
        "QuestReward"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::List
    }

    fn key(&self) -> Option<&'static str> {
        None
    }

    fn row_type(&self) -> &'static str {
        "quest_reward::QuestReward"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct GameSettingsTable {
    rows: game_settings::GameSettings,
}

impl GameSettingsTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_singleton_table(
                bundle.decode_table::<game_settings::GameSettings>("GameSettings")?,
                "GameSettings",
            )?,
        })
    }
}

impl std::ops::Deref for GameSettingsTable {
    type Target = game_settings::GameSettings;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for GameSettingsTable {
    fn name(&self) -> &'static str {
        "GameSettings"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Singleton
    }

    fn key(&self) -> Option<&'static str> {
        None
    }

    fn row_type(&self) -> &'static str {
        "game_settings::GameSettings"
    }

    fn len(&self) -> usize {
        1
    }
}

#[derive(Debug, Clone)]
pub struct LocalizationTable {
    rows: SoraMap<String, localization::Localization>,
}

impl LocalizationTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<localization::Localization>("Localization")?,
                |row| row.key.clone(),
            ),
        })
    }
    pub fn get(&self, key: &String) -> Option<&localization::Localization> {
        self.rows.get(key)
    }
}

impl std::ops::Deref for LocalizationTable {
    type Target = SoraMap<String, localization::Localization>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for LocalizationTable {
    fn name(&self) -> &'static str {
        "Localization"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("key")
    }

    fn row_type(&self) -> &'static str {
        "localization::Localization"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct LevelExpTable {
    rows: SoraMap<i32, level_exp::LevelExp>,
}

impl LevelExpTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<level_exp::LevelExp>("LevelExp")?,
                |row| row.level,
            ),
        })
    }
    pub fn get(&self, key: i32) -> Option<&level_exp::LevelExp> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for LevelExpTable {
    type Target = SoraMap<i32, level_exp::LevelExp>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for LevelExpTable {
    fn name(&self) -> &'static str {
        "LevelExp"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("level")
    }

    fn row_type(&self) -> &'static str {
        "level_exp::LevelExp"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct CharacterTable {
    rows: SoraMap<i32, character::Character>,
}

impl CharacterTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<character::Character>("Character")?,
                |row| row.id,
            ),
        })
    }
    pub fn get(&self, key: i32) -> Option<&character::Character> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for CharacterTable {
    type Target = SoraMap<i32, character::Character>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for CharacterTable {
    fn name(&self) -> &'static str {
        "Character"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "character::Character"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct CharacterSkillTable {
    rows: Vec<character_skill::CharacterSkill>,
}

impl CharacterSkillTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: bundle.decode_table::<character_skill::CharacterSkill>("CharacterSkill")?,
        })
    }
}

impl std::ops::Deref for CharacterSkillTable {
    type Target = Vec<character_skill::CharacterSkill>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for CharacterSkillTable {
    fn name(&self) -> &'static str {
        "CharacterSkill"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::List
    }

    fn key(&self) -> Option<&'static str> {
        None
    }

    fn row_type(&self) -> &'static str {
        "character_skill::CharacterSkill"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct BuffTable {
    rows: SoraMap<i32, buff::Buff>,
}

impl BuffTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(bundle.decode_table::<buff::Buff>("Buff")?, |row| row.id),
        })
    }
    pub fn get(&self, key: i32) -> Option<&buff::Buff> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for BuffTable {
    type Target = SoraMap<i32, buff::Buff>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for BuffTable {
    fn name(&self) -> &'static str {
        "Buff"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "buff::Buff"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct DropGroupTable {
    rows: SoraMap<i32, drop_group::DropGroup>,
}

impl DropGroupTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<drop_group::DropGroup>("DropGroup")?,
                |row| row.id,
            ),
        })
    }
    pub fn get(&self, key: i32) -> Option<&drop_group::DropGroup> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for DropGroupTable {
    type Target = SoraMap<i32, drop_group::DropGroup>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for DropGroupTable {
    fn name(&self) -> &'static str {
        "DropGroup"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "drop_group::DropGroup"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct DropEntryTable {
    rows: Vec<drop_entry::DropEntry>,
}

impl DropEntryTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: bundle.decode_table::<drop_entry::DropEntry>("DropEntry")?,
        })
    }
}

impl std::ops::Deref for DropEntryTable {
    type Target = Vec<drop_entry::DropEntry>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for DropEntryTable {
    fn name(&self) -> &'static str {
        "DropEntry"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::List
    }

    fn key(&self) -> Option<&'static str> {
        None
    }

    fn row_type(&self) -> &'static str {
        "drop_entry::DropEntry"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct MonsterTable {
    rows: SoraMap<i32, monster::Monster>,
}

impl MonsterTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(bundle.decode_table::<monster::Monster>("Monster")?, |row| {
                row.id
            }),
        })
    }
    pub fn get(&self, key: i32) -> Option<&monster::Monster> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for MonsterTable {
    type Target = SoraMap<i32, monster::Monster>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for MonsterTable {
    fn name(&self) -> &'static str {
        "Monster"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "monster::Monster"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct StageTable {
    rows: SoraMap<i32, stage::Stage>,
}

impl StageTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(bundle.decode_table::<stage::Stage>("Stage")?, |row| row.id),
        })
    }
    pub fn get(&self, key: i32) -> Option<&stage::Stage> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for StageTable {
    type Target = SoraMap<i32, stage::Stage>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for StageTable {
    fn name(&self) -> &'static str {
        "Stage"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "stage::Stage"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct StageRewardTable {
    rows: Vec<stage_reward::StageReward>,
}

impl StageRewardTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: bundle.decode_table::<stage_reward::StageReward>("StageReward")?,
        })
    }
}

impl std::ops::Deref for StageRewardTable {
    type Target = Vec<stage_reward::StageReward>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for StageRewardTable {
    fn name(&self) -> &'static str {
        "StageReward"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::List
    }

    fn key(&self) -> Option<&'static str> {
        None
    }

    fn row_type(&self) -> &'static str {
        "stage_reward::StageReward"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct DungeonTable {
    rows: SoraMap<i32, dungeon::Dungeon>,
}

impl DungeonTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(bundle.decode_table::<dungeon::Dungeon>("Dungeon")?, |row| {
                row.id
            }),
        })
    }
    pub fn get(&self, key: i32) -> Option<&dungeon::Dungeon> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for DungeonTable {
    type Target = SoraMap<i32, dungeon::Dungeon>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for DungeonTable {
    fn name(&self) -> &'static str {
        "Dungeon"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "dungeon::Dungeon"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct ShopTable {
    rows: SoraMap<i32, shop::Shop>,
}

impl ShopTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(bundle.decode_table::<shop::Shop>("Shop")?, |row| row.id),
        })
    }
    pub fn get(&self, key: i32) -> Option<&shop::Shop> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for ShopTable {
    type Target = SoraMap<i32, shop::Shop>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for ShopTable {
    fn name(&self) -> &'static str {
        "Shop"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "shop::Shop"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct ShopItemTable {
    rows: Vec<shop_item::ShopItem>,
}

impl ShopItemTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: bundle.decode_table::<shop_item::ShopItem>("ShopItem")?,
        })
    }
}

impl std::ops::Deref for ShopItemTable {
    type Target = Vec<shop_item::ShopItem>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for ShopItemTable {
    fn name(&self) -> &'static str {
        "ShopItem"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::List
    }

    fn key(&self) -> Option<&'static str> {
        None
    }

    fn row_type(&self) -> &'static str {
        "shop_item::ShopItem"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct RecipeTable {
    rows: SoraMap<i32, recipe::Recipe>,
}

impl RecipeTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(bundle.decode_table::<recipe::Recipe>("Recipe")?, |row| {
                row.id
            }),
        })
    }
    pub fn get(&self, key: i32) -> Option<&recipe::Recipe> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for RecipeTable {
    type Target = SoraMap<i32, recipe::Recipe>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for RecipeTable {
    fn name(&self) -> &'static str {
        "Recipe"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "recipe::Recipe"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct GachaPoolTable {
    rows: SoraMap<i32, gacha_pool::GachaPool>,
}

impl GachaPoolTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<gacha_pool::GachaPool>("GachaPool")?,
                |row| row.id,
            ),
        })
    }
    pub fn get(&self, key: i32) -> Option<&gacha_pool::GachaPool> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for GachaPoolTable {
    type Target = SoraMap<i32, gacha_pool::GachaPool>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for GachaPoolTable {
    fn name(&self) -> &'static str {
        "GachaPool"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "gacha_pool::GachaPool"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct GachaItemTable {
    rows: Vec<gacha_item::GachaItem>,
}

impl GachaItemTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: bundle.decode_table::<gacha_item::GachaItem>("GachaItem")?,
        })
    }
}

impl std::ops::Deref for GachaItemTable {
    type Target = Vec<gacha_item::GachaItem>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for GachaItemTable {
    fn name(&self) -> &'static str {
        "GachaItem"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::List
    }

    fn key(&self) -> Option<&'static str> {
        None
    }

    fn row_type(&self) -> &'static str {
        "gacha_item::GachaItem"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct EquipmentSetTable {
    rows: SoraMap<i32, equipment_set::EquipmentSet>,
}

impl EquipmentSetTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<equipment_set::EquipmentSet>("EquipmentSet")?,
                |row| row.id,
            ),
        })
    }
    pub fn get(&self, key: i32) -> Option<&equipment_set::EquipmentSet> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for EquipmentSetTable {
    type Target = SoraMap<i32, equipment_set::EquipmentSet>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for EquipmentSetTable {
    fn name(&self) -> &'static str {
        "EquipmentSet"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "equipment_set::EquipmentSet"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct AchievementTable {
    rows: SoraMap<i32, achievement::Achievement>,
}

impl AchievementTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<achievement::Achievement>("Achievement")?,
                |row| row.id,
            ),
        })
    }
    pub fn get(&self, key: i32) -> Option<&achievement::Achievement> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for AchievementTable {
    type Target = SoraMap<i32, achievement::Achievement>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for AchievementTable {
    fn name(&self) -> &'static str {
        "Achievement"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "achievement::Achievement"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct VipLevelTable {
    rows: SoraMap<i32, vip_level::VipLevel>,
}

impl VipLevelTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<vip_level::VipLevel>("VipLevel")?,
                |row| row.level,
            ),
        })
    }
    pub fn get(&self, key: i32) -> Option<&vip_level::VipLevel> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for VipLevelTable {
    type Target = SoraMap<i32, vip_level::VipLevel>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for VipLevelTable {
    fn name(&self) -> &'static str {
        "VipLevel"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("level")
    }

    fn row_type(&self) -> &'static str {
        "vip_level::VipLevel"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct MailTemplateTable {
    rows: SoraMap<i32, mail_template::MailTemplate>,
}

impl MailTemplateTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<mail_template::MailTemplate>("MailTemplate")?,
                |row| row.id,
            ),
        })
    }
    pub fn get(&self, key: i32) -> Option<&mail_template::MailTemplate> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for MailTemplateTable {
    type Target = SoraMap<i32, mail_template::MailTemplate>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for MailTemplateTable {
    fn name(&self) -> &'static str {
        "MailTemplate"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "mail_template::MailTemplate"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct MailRewardTable {
    rows: Vec<mail_reward::MailReward>,
}

impl MailRewardTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: bundle.decode_table::<mail_reward::MailReward>("MailReward")?,
        })
    }
}

impl std::ops::Deref for MailRewardTable {
    type Target = Vec<mail_reward::MailReward>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for MailRewardTable {
    fn name(&self) -> &'static str {
        "MailReward"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::List
    }

    fn key(&self) -> Option<&'static str> {
        None
    }

    fn row_type(&self) -> &'static str {
        "mail_reward::MailReward"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct DialogueTable {
    rows: SoraMap<i32, dialogue::Dialogue>,
}

impl DialogueTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<dialogue::Dialogue>("Dialogue")?,
                |row| row.id,
            ),
        })
    }
    pub fn get(&self, key: i32) -> Option<&dialogue::Dialogue> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for DialogueTable {
    type Target = SoraMap<i32, dialogue::Dialogue>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for DialogueTable {
    fn name(&self) -> &'static str {
        "Dialogue"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "dialogue::Dialogue"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug, Clone)]
pub struct EventRuleTable {
    rows: SoraMap<i32, event_rule::EventRule>,
}

impl EventRuleTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self {
            rows: decode_map_table(
                bundle.decode_table::<event_rule::EventRule>("EventRule")?,
                |row| row.id,
            ),
        })
    }
    pub fn get(&self, key: i32) -> Option<&event_rule::EventRule> {
        self.rows.get(&key)
    }
}

impl std::ops::Deref for EventRuleTable {
    type Target = SoraMap<i32, event_rule::EventRule>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl SoraTable for EventRuleTable {
    fn name(&self) -> &'static str {
        "EventRule"
    }

    fn mode(&self) -> SoraTableMode {
        SoraTableMode::Map
    }

    fn key(&self) -> Option<&'static str> {
        Some("id")
    }

    fn row_type(&self) -> &'static str {
        "event_rule::EventRule"
    }

    fn len(&self) -> usize {
        self.rows.len()
    }
}

impl SoraConfig {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, runtime::SoraReadError> {
        let bundle = runtime::SoraBundle::parse(bytes)?;
        let mut tables: SoraMap<&'static str, Box<dyn SoraTable>> = sora_map_with_capacity(28);
        tables.insert("Item", Box::new(ItemTable::decode(&bundle)?));
        tables.insert("Skill", Box::new(SkillTable::decode(&bundle)?));
        tables.insert("Quest", Box::new(QuestTable::decode(&bundle)?));
        tables.insert("QuestReward", Box::new(QuestRewardTable::decode(&bundle)?));
        tables.insert(
            "GameSettings",
            Box::new(GameSettingsTable::decode(&bundle)?),
        );
        tables.insert(
            "Localization",
            Box::new(LocalizationTable::decode(&bundle)?),
        );
        tables.insert("LevelExp", Box::new(LevelExpTable::decode(&bundle)?));
        tables.insert("Character", Box::new(CharacterTable::decode(&bundle)?));
        tables.insert(
            "CharacterSkill",
            Box::new(CharacterSkillTable::decode(&bundle)?),
        );
        tables.insert("Buff", Box::new(BuffTable::decode(&bundle)?));
        tables.insert("DropGroup", Box::new(DropGroupTable::decode(&bundle)?));
        tables.insert("DropEntry", Box::new(DropEntryTable::decode(&bundle)?));
        tables.insert("Monster", Box::new(MonsterTable::decode(&bundle)?));
        tables.insert("Stage", Box::new(StageTable::decode(&bundle)?));
        tables.insert("StageReward", Box::new(StageRewardTable::decode(&bundle)?));
        tables.insert("Dungeon", Box::new(DungeonTable::decode(&bundle)?));
        tables.insert("Shop", Box::new(ShopTable::decode(&bundle)?));
        tables.insert("ShopItem", Box::new(ShopItemTable::decode(&bundle)?));
        tables.insert("Recipe", Box::new(RecipeTable::decode(&bundle)?));
        tables.insert("GachaPool", Box::new(GachaPoolTable::decode(&bundle)?));
        tables.insert("GachaItem", Box::new(GachaItemTable::decode(&bundle)?));
        tables.insert(
            "EquipmentSet",
            Box::new(EquipmentSetTable::decode(&bundle)?),
        );
        tables.insert("Achievement", Box::new(AchievementTable::decode(&bundle)?));
        tables.insert("VipLevel", Box::new(VipLevelTable::decode(&bundle)?));
        tables.insert(
            "MailTemplate",
            Box::new(MailTemplateTable::decode(&bundle)?),
        );
        tables.insert("MailReward", Box::new(MailRewardTable::decode(&bundle)?));
        tables.insert("Dialogue", Box::new(DialogueTable::decode(&bundle)?));
        tables.insert("EventRule", Box::new(EventRuleTable::decode(&bundle)?));
        Ok(Self { tables })
    }

    fn table<T: SoraTable + 'static>(&self, name: &'static str) -> &T {
        self.tables
            .get(name)
            .and_then(|table| {
                let table: &dyn std::any::Any = table.as_ref();
                table.downcast_ref::<T>()
            })
            .unwrap_or_else(|| {
                panic!(
                    "generated SoraConfig is missing table `{}` or has an unexpected table type",
                    name
                )
            })
    }

    pub fn tables(&self) -> impl Iterator<Item = &dyn SoraTable> {
        self.tables.values().map(Box::as_ref)
    }

    pub fn item(&self) -> &ItemTable {
        self.table("Item")
    }

    pub fn skill(&self) -> &SkillTable {
        self.table("Skill")
    }

    pub fn quest(&self) -> &QuestTable {
        self.table("Quest")
    }

    pub fn quest_reward(&self) -> &QuestRewardTable {
        self.table("QuestReward")
    }

    pub fn game_settings(&self) -> &GameSettingsTable {
        self.table("GameSettings")
    }

    pub fn localization(&self) -> &LocalizationTable {
        self.table("Localization")
    }

    pub fn level_exp(&self) -> &LevelExpTable {
        self.table("LevelExp")
    }

    pub fn character(&self) -> &CharacterTable {
        self.table("Character")
    }

    pub fn character_skill(&self) -> &CharacterSkillTable {
        self.table("CharacterSkill")
    }

    pub fn buff(&self) -> &BuffTable {
        self.table("Buff")
    }

    pub fn drop_group(&self) -> &DropGroupTable {
        self.table("DropGroup")
    }

    pub fn drop_entry(&self) -> &DropEntryTable {
        self.table("DropEntry")
    }

    pub fn monster(&self) -> &MonsterTable {
        self.table("Monster")
    }

    pub fn stage(&self) -> &StageTable {
        self.table("Stage")
    }

    pub fn stage_reward(&self) -> &StageRewardTable {
        self.table("StageReward")
    }

    pub fn dungeon(&self) -> &DungeonTable {
        self.table("Dungeon")
    }

    pub fn shop(&self) -> &ShopTable {
        self.table("Shop")
    }

    pub fn shop_item(&self) -> &ShopItemTable {
        self.table("ShopItem")
    }

    pub fn recipe(&self) -> &RecipeTable {
        self.table("Recipe")
    }

    pub fn gacha_pool(&self) -> &GachaPoolTable {
        self.table("GachaPool")
    }

    pub fn gacha_item(&self) -> &GachaItemTable {
        self.table("GachaItem")
    }

    pub fn equipment_set(&self) -> &EquipmentSetTable {
        self.table("EquipmentSet")
    }

    pub fn achievement(&self) -> &AchievementTable {
        self.table("Achievement")
    }

    pub fn vip_level(&self) -> &VipLevelTable {
        self.table("VipLevel")
    }

    pub fn mail_template(&self) -> &MailTemplateTable {
        self.table("MailTemplate")
    }

    pub fn mail_reward(&self) -> &MailRewardTable {
        self.table("MailReward")
    }

    pub fn dialogue(&self) -> &DialogueTable {
        self.table("Dialogue")
    }

    pub fn event_rule(&self) -> &EventRuleTable {
        self.table("EventRule")
    }
}

fn sora_map_with_capacity<K, V>(capacity: usize) -> SoraMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    SoraMap::with_capacity_and_hasher(capacity, Default::default())
}

fn decode_map_table<K, V>(rows: Vec<V>, key: impl Fn(&V) -> K) -> SoraMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    rows.into_iter().map(|row| (key(&row), row)).collect()
}

fn build_unique_list_index<'a, K, V: 'a>(
    rows: impl Iterator<Item = &'a V>,
    key: impl Fn(&V) -> K,
) -> SoraMap<K, usize>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    let mut index = sora_map_with_capacity(rows.size_hint().0);
    for (position, row) in rows.enumerate() {
        index.insert(key(row), position);
    }
    index
}

fn build_unique_map_index<'a, K, P, V: 'a>(
    rows: impl Iterator<Item = &'a V>,
    key: impl Fn(&V) -> K,
    primary_key: impl Fn(&V) -> P,
) -> SoraMap<K, P>
where
    K: std::cmp::Eq + std::hash::Hash,
    P: std::cmp::Eq + std::hash::Hash,
{
    let mut index = sora_map_with_capacity(rows.size_hint().0);
    for row in rows {
        index.insert(key(row), primary_key(row));
    }
    index
}

fn decode_singleton_table<T>(mut rows: Vec<T>, name: &str) -> Result<T, runtime::SoraReadError> {
    if rows.len() != 1 {
        return Err(runtime::SoraReadError::new(format!(
            "expected singleton table `{}` to contain exactly 1 row, got {}",
            name,
            rows.len()
        )));
    }

    Ok(rows.remove(0))
}
