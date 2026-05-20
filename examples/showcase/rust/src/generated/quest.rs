
use super::quest_type::QuestType;
use super::reward::Reward;
use super::vec3::Vec3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Quest {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "quest_type")]
    pub quest_type: QuestType,
    #[serde(rename = "title")]
    pub title: String,
    #[serde(rename = "required_item")]
    pub required_item: i32,
    #[serde(rename = "unlock_skills")]
    pub unlock_skills: Vec<i32>,
    #[serde(rename = "start_pos")]
    pub start_pos: Vec3,
    /// Materialized from QuestReward child rows
    #[serde(rename = "rewards")]
    pub rewards: Vec<Reward>,
}

impl super::runtime::SoraDecode for Quest {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            quest_type: <QuestType as super::runtime::SoraDecode>::decode(reader)?,
            title: <String as super::runtime::SoraDecode>::decode(reader)?,
            required_item: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            unlock_skills: <Vec<i32> as super::runtime::SoraDecode>::decode(reader)?,
            start_pos: <Vec3 as super::runtime::SoraDecode>::decode(reader)?,
            rewards: <Vec<Reward> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
