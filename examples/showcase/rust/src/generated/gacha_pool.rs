use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GachaPool {
    pub id: i32,
    pub name: String,
    pub cost: ResourceCost,
}

impl super::runtime::SoraDecode for GachaPool {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            cost: <ResourceCost as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for GachaPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("GachaPool");
        builder.field("id", &self.id);
        builder.field("name", &self.name);
        builder.field("cost", &self.cost);
        builder.finish()
    }
}
