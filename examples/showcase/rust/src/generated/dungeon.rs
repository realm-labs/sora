use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dungeon {
    pub id: i32,
    pub name: String,
    pub stage_ids: Vec<i32>,
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

impl std::fmt::Display for Dungeon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Dungeon");
        builder.field("id", &self.id);
        builder.field("name", &self.name);
        builder.field("stage_ids", &self.stage_ids);
        builder.field("entry_cost", &self.entry_cost);
        builder.finish()
    }
}
