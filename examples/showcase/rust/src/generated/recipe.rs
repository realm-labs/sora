use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Recipe {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "result_item")]
    pub result_item: i32,
    #[serde(rename = "materials")]
    pub materials: Vec<ResourceCost>,
}

impl super::runtime::SoraDecode for Recipe {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            result_item: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            materials: <Vec<ResourceCost> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
