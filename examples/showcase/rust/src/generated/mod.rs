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

#[derive(Debug, Clone)]
pub struct SoraConfig {
    pub item: std::collections::HashMap<i32, item::Item>,
    pub skill: std::collections::HashMap<i32, skill::Skill>,
    pub quest: std::collections::HashMap<i32, quest::Quest>,
    pub quest_reward: Vec<quest_reward::QuestReward>,
    pub game_settings: game_settings::GameSettings,
}

impl SoraConfig {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, runtime::SoraReadError> {
        let bundle = runtime::SoraBundle::parse(bytes)?;
        Ok(Self {
            item: decode_map_table(bundle.decode_table::<item::Item>("Item")?, |row| row.id),
            skill: decode_map_table(bundle.decode_table::<skill::Skill>("Skill")?, |row| row.id),
            quest: decode_map_table(bundle.decode_table::<quest::Quest>("Quest")?, |row| row.id),
            quest_reward: bundle.decode_table::<quest_reward::QuestReward>("QuestReward")?,
            game_settings: decode_singleton_table(
                bundle.decode_table::<game_settings::GameSettings>("GameSettings")?,
                "GameSettings",
            )?,
        })
    }
    pub fn get_item(&self, key: i32) -> Option<&item::Item> {
        self.item.get(&key)
    }

    pub fn iter_item(&self) -> impl Iterator<Item = &item::Item> {
        self.item.values()
    }
    pub fn get_skill(&self, key: i32) -> Option<&skill::Skill> {
        self.skill.get(&key)
    }

    pub fn iter_skill(&self) -> impl Iterator<Item = &skill::Skill> {
        self.skill.values()
    }
    pub fn get_quest(&self, key: i32) -> Option<&quest::Quest> {
        self.quest.get(&key)
    }

    pub fn iter_quest(&self) -> impl Iterator<Item = &quest::Quest> {
        self.quest.values()
    }

    pub fn quest_reward_rows(&self) -> &[quest_reward::QuestReward] {
        &self.quest_reward
    }

    pub fn iter_quest_reward(&self) -> impl Iterator<Item = &quest_reward::QuestReward> {
        self.quest_reward.iter()
    }

    pub fn game_settings_row(&self) -> &game_settings::GameSettings {
        &self.game_settings
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
