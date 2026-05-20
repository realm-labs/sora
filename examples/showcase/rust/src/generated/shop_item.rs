use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShopItem {
    pub shop_id: i32,
    pub seq: i32,
    pub item_id: i32,
    pub price: ResourceCost,
    pub daily_limit: Option<i32>,
}

impl super::runtime::SoraDecode for ShopItem {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            shop_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            seq: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            price: <ResourceCost as super::runtime::SoraDecode>::decode(reader)?,
            daily_limit: <Option<i32> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
