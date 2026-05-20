#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EventCondition {
    LevelAtLeast { level: i32 },
    QuestCompleted { quest_id: i32 },
    HasItem { item_id: i32, count: i32 },
}

impl super::runtime::SoraDecode for EventCondition {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        match reader.read_u32()? {
            0 => Ok(Self::LevelAtLeast {
                level: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            }),
            1 => Ok(Self::QuestCompleted {
                quest_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            }),
            2 => Ok(Self::HasItem {
                item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
                count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            }),
            value => Err(super::runtime::SoraReadError::new(format!(
                "invalid union ordinal {} for EventCondition",
                value
            ))),
        }
    }
}

impl std::fmt::Display for EventCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LevelAtLeast { level } => {
                let mut builder = f.debug_struct("EventCondition::LevelAtLeast");
                builder.field("level", level);
                builder.finish()
            }
            Self::QuestCompleted { quest_id } => {
                let mut builder = f.debug_struct("EventCondition::QuestCompleted");
                builder.field("quest_id", quest_id);
                builder.finish()
            }
            Self::HasItem { item_id, count } => {
                let mut builder = f.debug_struct("EventCondition::HasItem");
                builder.field("item_id", item_id);
                builder.field("count", count);
                builder.finish()
            }
        }
    }
}
