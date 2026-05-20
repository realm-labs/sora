use super::rarity::Rarity;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GachaItem {
    pub pool_id: i32,
    pub item_id: i32,
    pub rarity: Rarity,
    pub weight: f32,
}

impl super::runtime::SoraDecode for GachaItem {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            pool_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            rarity: <Rarity as super::runtime::SoraDecode>::decode(reader)?,
            weight: <f32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for GachaItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("GachaItem");
        builder.field("pool_id", &self.pool_id);
        builder.field("item_id", &self.item_id);
        builder.field("rarity", &self.rarity);
        builder.field("weight", &self.weight);
        builder.finish()
    }
}
