use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Recipe {
    pub id: i32,
    pub result_item: i32,
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
