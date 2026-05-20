use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VipLevel {
    #[serde(rename = "level")]
    pub level: i32,
    #[serde(rename = "cost")]
    pub cost: ResourceCost,
    #[serde(rename = "perks")]
    pub perks: Vec<String>,
}

impl super::runtime::SoraDecode for VipLevel {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            level: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            cost: <ResourceCost as super::runtime::SoraDecode>::decode(reader)?,
            perks: <Vec<String> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
