use super::item_type::ItemType;
use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Item {
    /// Item id
    #[serde(rename = "id")]
    pub id: i32,
    /// Display name
    #[serde(rename = "name")]
    pub name: String,
    /// Item category
    #[serde(rename = "item_type")]
    pub item_type: ItemType,
    /// Stack limit; blank cells use the default
    #[serde(rename = "max_stack")]
    pub max_stack: i32,
    /// Tuple: kind,id,count
    #[serde(rename = "price")]
    pub price: ResourceCost,
    /// JSON string array
    #[serde(rename = "tags")]
    pub tags: Vec<String>,
}

impl super::runtime::SoraDecode for Item {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            item_type: <ItemType as super::runtime::SoraDecode>::decode(reader)?,
            max_stack: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            price: <ResourceCost as super::runtime::SoraDecode>::decode(reader)?,
            tags: <Vec<String> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
