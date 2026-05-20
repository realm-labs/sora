use super::item_type::ItemType;
use super::resource_cost::ResourceCost;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Item {
    /// Item id
    pub id: i32,
    /// Display name
    pub name: String,
    /// Item category
    pub item_type: ItemType,
    /// Stack limit; blank cells use the default
    pub max_stack: i32,
    /// Tuple: kind,id,count
    pub price: ResourceCost,
    /// JSON string array
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

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Item");
        builder.field("id", &self.id);
        builder.field("name", &self.name);
        builder.field("item_type", &self.item_type);
        builder.field("max_stack", &self.max_stack);
        builder.field("price", &self.price);
        builder.field("tags", &self.tags);
        builder.finish()
    }
}
