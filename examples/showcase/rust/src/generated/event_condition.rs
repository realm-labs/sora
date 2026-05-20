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
