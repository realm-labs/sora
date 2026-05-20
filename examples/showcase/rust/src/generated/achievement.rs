use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Achievement {
    pub id: i32,
    pub title_key: String,
    pub target_count: i64,
    pub reward: ResourceCost,
}

impl super::runtime::SoraDecode for Achievement {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            title_key: <String as super::runtime::SoraDecode>::decode(reader)?,
            target_count: <i64 as super::runtime::SoraDecode>::decode(reader)?,
            reward: <ResourceCost as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for Achievement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Achievement");
        builder.field("id", &self.id);
        builder.field("title_key", &self.title_key);
        builder.field("target_count", &self.target_count);
        builder.field("reward", &self.reward);
        builder.finish()
    }
}
