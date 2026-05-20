#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Reward {
    pub item_id: i32,
    pub count: i32,
}

impl super::runtime::SoraDecode for Reward {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for Reward {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Reward");
        builder.field("item_id", &self.item_id);
        builder.field("count", &self.count);
        builder.finish()
    }
}
