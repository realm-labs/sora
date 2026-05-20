#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StageReward {
    pub stage_id: i32,
    pub seq: i32,
    pub item_id: i32,
    pub count: i32,
}

impl super::runtime::SoraDecode for StageReward {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            stage_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            seq: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
