pub mod element_type;
pub mod game_settings;
pub mod item;
pub mod item_type;
pub mod quest;
pub mod quest_reward;
pub mod quest_type;
pub mod resource_cost;
pub mod resource_kind;
pub mod reward;
pub mod runtime;
pub mod skill;
pub mod skill_effect;
pub mod vec3;

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

impl SoraConfig {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, runtime::SoraReadError> {
        let bundle = runtime::SoraBundle::parse(bytes)?;
        let mut tables: std::collections::HashMap<
            &'static str,
            Box<dyn std::any::Any + Send + Sync>,
        > = std::collections::HashMap::with_capacity(5);
        tables.insert("Item", Box::new(ItemTable::decode(&bundle)?));
        tables.insert("Skill", Box::new(SkillTable::decode(&bundle)?));
        tables.insert("Quest", Box::new(QuestTable::decode(&bundle)?));
        tables.insert("QuestReward", Box::new(QuestRewardTable::decode(&bundle)?));
        tables.insert(
            "GameSettings",
            Box::new(GameSettingsTable::decode(&bundle)?),
        );
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
