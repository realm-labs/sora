

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StageReward {
    #[serde(rename = "stage_id")]
    pub stage_id: i32,
    #[serde(rename = "seq")]
    pub seq: i32,
    #[serde(rename = "item_id")]
    pub item_id: i32,
    #[serde(rename = "count")]
    pub count: i32,
}

impl super::runtime::SoraDecode for StageReward {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            stage_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            seq: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
