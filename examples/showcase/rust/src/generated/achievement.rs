
use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Achievement {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "title_key")]
    pub title_key: String,
    #[serde(rename = "target_count")]
    pub target_count: i64,
    #[serde(rename = "reward")]
    pub reward: ResourceCost,
}

impl super::runtime::SoraDecode for Achievement {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            title_key: <String as super::runtime::SoraDecode>::decode(reader)?,
            target_count: <i64 as super::runtime::SoraDecode>::decode(reader)?,
            reward: <ResourceCost as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
