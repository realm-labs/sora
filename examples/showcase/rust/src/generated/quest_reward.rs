#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuestReward {
    #[serde(rename = "quest_id")]
    pub quest_id: i32,
    #[serde(rename = "seq")]
    pub seq: i32,
    #[serde(rename = "item_id")]
    pub item_id: i32,
    #[serde(rename = "count")]
    pub count: i32,
}

impl super::runtime::SoraDecode for QuestReward {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            quest_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            seq: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
