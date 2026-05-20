use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dungeon {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "stage_ids")]
    pub stage_ids: Vec<i32>,
    #[serde(rename = "entry_cost")]
    pub entry_cost: ResourceCost,
}

impl super::runtime::SoraDecode for Dungeon {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            stage_ids: <Vec<i32> as super::runtime::SoraDecode>::decode(reader)?,
            entry_cost: <ResourceCost as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
