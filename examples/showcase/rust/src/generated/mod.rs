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

pub struct SoraConfig {
    tables: std::collections::HashMap<&'static str, Box<dyn std::any::Any + Send + Sync>>,
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
pub struct ItemTable(std::collections::HashMap<i32, item::Item>);

impl ItemTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<item::Item>("Item")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&item::Item> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for ItemTable {
    type Target = std::collections::HashMap<i32, item::Item>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct SkillTable(std::collections::HashMap<i32, skill::Skill>);

impl SkillTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<skill::Skill>("Skill")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&skill::Skill> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for SkillTable {
    type Target = std::collections::HashMap<i32, skill::Skill>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct QuestTable(std::collections::HashMap<i32, quest::Quest>);

impl QuestTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<quest::Quest>("Quest")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&quest::Quest> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for QuestTable {
    type Target = std::collections::HashMap<i32, quest::Quest>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct QuestRewardTable(Vec<quest_reward::QuestReward>);

impl QuestRewardTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(bundle.decode_table::<quest_reward::QuestReward>(
            "QuestReward",
        )?))
    }
}

impl std::ops::Deref for QuestRewardTable {
    type Target = Vec<quest_reward::QuestReward>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct GameSettingsTable(game_settings::GameSettings);

impl GameSettingsTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_singleton_table(
            bundle.decode_table::<game_settings::GameSettings>("GameSettings")?,
            "GameSettings",
        )?))
    }
}

impl std::ops::Deref for GameSettingsTable {
    type Target = game_settings::GameSettings;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct LocalizationTable(std::collections::HashMap<String, localization::Localization>);

impl LocalizationTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<localization::Localization>("Localization")?,
            |row| row.key.clone(),
        )))
    }
    pub fn get(&self, key: &String) -> Option<&localization::Localization> {
        self.0.get(key)
    }
}

impl std::ops::Deref for LocalizationTable {
    type Target = std::collections::HashMap<String, localization::Localization>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct LevelExpTable(std::collections::HashMap<i32, level_exp::LevelExp>);

impl LevelExpTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<level_exp::LevelExp>("LevelExp")?,
            |row| row.level,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&level_exp::LevelExp> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for LevelExpTable {
    type Target = std::collections::HashMap<i32, level_exp::LevelExp>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct CharacterTable(std::collections::HashMap<i32, character::Character>);

impl CharacterTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<character::Character>("Character")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&character::Character> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for CharacterTable {
    type Target = std::collections::HashMap<i32, character::Character>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct CharacterSkillTable(Vec<character_skill::CharacterSkill>);

impl CharacterSkillTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(
            bundle.decode_table::<character_skill::CharacterSkill>("CharacterSkill")?,
        ))
    }
}

impl std::ops::Deref for CharacterSkillTable {
    type Target = Vec<character_skill::CharacterSkill>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct BuffTable(std::collections::HashMap<i32, buff::Buff>);

impl BuffTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<buff::Buff>("Buff")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&buff::Buff> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for BuffTable {
    type Target = std::collections::HashMap<i32, buff::Buff>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct DropGroupTable(std::collections::HashMap<i32, drop_group::DropGroup>);

impl DropGroupTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<drop_group::DropGroup>("DropGroup")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&drop_group::DropGroup> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for DropGroupTable {
    type Target = std::collections::HashMap<i32, drop_group::DropGroup>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct DropEntryTable(Vec<drop_entry::DropEntry>);

impl DropEntryTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(
            bundle.decode_table::<drop_entry::DropEntry>("DropEntry")?,
        ))
    }
}

impl std::ops::Deref for DropEntryTable {
    type Target = Vec<drop_entry::DropEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct MonsterTable(std::collections::HashMap<i32, monster::Monster>);

impl MonsterTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<monster::Monster>("Monster")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&monster::Monster> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for MonsterTable {
    type Target = std::collections::HashMap<i32, monster::Monster>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct StageTable(std::collections::HashMap<i32, stage::Stage>);

impl StageTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<stage::Stage>("Stage")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&stage::Stage> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for StageTable {
    type Target = std::collections::HashMap<i32, stage::Stage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct StageRewardTable(Vec<stage_reward::StageReward>);

impl StageRewardTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(bundle.decode_table::<stage_reward::StageReward>(
            "StageReward",
        )?))
    }
}

impl std::ops::Deref for StageRewardTable {
    type Target = Vec<stage_reward::StageReward>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct DungeonTable(std::collections::HashMap<i32, dungeon::Dungeon>);

impl DungeonTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<dungeon::Dungeon>("Dungeon")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&dungeon::Dungeon> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for DungeonTable {
    type Target = std::collections::HashMap<i32, dungeon::Dungeon>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ShopTable(std::collections::HashMap<i32, shop::Shop>);

impl ShopTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<shop::Shop>("Shop")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&shop::Shop> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for ShopTable {
    type Target = std::collections::HashMap<i32, shop::Shop>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ShopItemTable(Vec<shop_item::ShopItem>);

impl ShopItemTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(
            bundle.decode_table::<shop_item::ShopItem>("ShopItem")?,
        ))
    }
}

impl std::ops::Deref for ShopItemTable {
    type Target = Vec<shop_item::ShopItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct RecipeTable(std::collections::HashMap<i32, recipe::Recipe>);

impl RecipeTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<recipe::Recipe>("Recipe")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&recipe::Recipe> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for RecipeTable {
    type Target = std::collections::HashMap<i32, recipe::Recipe>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct GachaPoolTable(std::collections::HashMap<i32, gacha_pool::GachaPool>);

impl GachaPoolTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<gacha_pool::GachaPool>("GachaPool")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&gacha_pool::GachaPool> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for GachaPoolTable {
    type Target = std::collections::HashMap<i32, gacha_pool::GachaPool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct GachaItemTable(Vec<gacha_item::GachaItem>);

impl GachaItemTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(
            bundle.decode_table::<gacha_item::GachaItem>("GachaItem")?,
        ))
    }
}

impl std::ops::Deref for GachaItemTable {
    type Target = Vec<gacha_item::GachaItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct EquipmentSetTable(std::collections::HashMap<i32, equipment_set::EquipmentSet>);

impl EquipmentSetTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<equipment_set::EquipmentSet>("EquipmentSet")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&equipment_set::EquipmentSet> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for EquipmentSetTable {
    type Target = std::collections::HashMap<i32, equipment_set::EquipmentSet>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct AchievementTable(std::collections::HashMap<i32, achievement::Achievement>);

impl AchievementTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<achievement::Achievement>("Achievement")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&achievement::Achievement> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for AchievementTable {
    type Target = std::collections::HashMap<i32, achievement::Achievement>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct VipLevelTable(std::collections::HashMap<i32, vip_level::VipLevel>);

impl VipLevelTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<vip_level::VipLevel>("VipLevel")?,
            |row| row.level,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&vip_level::VipLevel> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for VipLevelTable {
    type Target = std::collections::HashMap<i32, vip_level::VipLevel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct MailTemplateTable(std::collections::HashMap<i32, mail_template::MailTemplate>);

impl MailTemplateTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<mail_template::MailTemplate>("MailTemplate")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&mail_template::MailTemplate> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for MailTemplateTable {
    type Target = std::collections::HashMap<i32, mail_template::MailTemplate>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct MailRewardTable(Vec<mail_reward::MailReward>);

impl MailRewardTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(
            bundle.decode_table::<mail_reward::MailReward>("MailReward")?,
        ))
    }
}

impl std::ops::Deref for MailRewardTable {
    type Target = Vec<mail_reward::MailReward>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct DialogueTable(std::collections::HashMap<i32, dialogue::Dialogue>);

impl DialogueTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<dialogue::Dialogue>("Dialogue")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&dialogue::Dialogue> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for DialogueTable {
    type Target = std::collections::HashMap<i32, dialogue::Dialogue>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct EventRuleTable(std::collections::HashMap<i32, event_rule::EventRule>);

impl EventRuleTable {
    fn decode(bundle: &runtime::SoraBundle<'_>) -> Result<Self, runtime::SoraReadError> {
        Ok(Self(decode_map_table(
            bundle.decode_table::<event_rule::EventRule>("EventRule")?,
            |row| row.id,
        )))
    }
    pub fn get(&self, key: i32) -> Option<&event_rule::EventRule> {
        self.0.get(&key)
    }
}

impl std::ops::Deref for EventRuleTable {
    type Target = std::collections::HashMap<i32, event_rule::EventRule>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SoraConfig {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, runtime::SoraReadError> {
        let bundle = runtime::SoraBundle::parse(bytes)?;
        let mut tables: std::collections::HashMap<
            &'static str,
            Box<dyn std::any::Any + Send + Sync>,
        > = std::collections::HashMap::with_capacity(28);
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

    fn table<T: 'static>(&self, name: &'static str) -> &T {
        self.tables
            .get(name)
            .and_then(|table| table.as_ref().downcast_ref::<T>())
            .unwrap_or_else(|| {
                panic!(
                    "generated SoraConfig is missing table `{}` or has an unexpected table type",
                    name
                )
            })
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

fn decode_map_table<K, V>(rows: Vec<V>, key: impl Fn(&V) -> K) -> std::collections::HashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash,
{
    rows.into_iter().map(|row| (key(&row), row)).collect()
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
