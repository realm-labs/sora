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

impl std::fmt::Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Recipe");
        builder.field("id", &self.id);
        builder.field("result_item", &self.result_item);
        builder.field("materials", &self.materials);
        builder.finish()
    }
}
