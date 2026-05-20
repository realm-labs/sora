use super::quest_type::QuestType;
use super::reward::Reward;
use super::vec3::Vec3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Quest {
    pub id: i32,
    pub quest_type: QuestType,
    pub title: String,
    pub required_item: i32,
    pub unlock_skills: Vec<i32>,
    pub start_pos: Vec3,
    /// Materialized from QuestReward child rows
    pub rewards: Vec<Reward>,
}

impl super::runtime::SoraDecode for Quest {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
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

impl std::fmt::Display for Quest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Quest");
        builder.field("id", &self.id);
        builder.field("quest_type", &self.quest_type);
        builder.field("title", &self.title);
        builder.field("required_item", &self.required_item);
        builder.field("unlock_skills", &self.unlock_skills);
        builder.field("start_pos", &self.start_pos);
        builder.field("rewards", &self.rewards);
        builder.finish()
    }
}
