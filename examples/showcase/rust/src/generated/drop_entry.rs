

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DropEntry {
    #[serde(rename = "group_id")]
    pub group_id: i32,
    #[serde(rename = "seq")]
    pub seq: i32,
    #[serde(rename = "item_id")]
    pub item_id: i32,
    #[serde(rename = "count")]
    pub count: i32,
    #[serde(rename = "weight")]
    pub weight: f32,
}

impl super::runtime::SoraDecode for DropEntry {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            group_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            seq: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            weight: <f32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}