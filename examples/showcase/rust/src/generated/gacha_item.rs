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
