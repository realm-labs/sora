#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DropGroup {
    pub id: i32,
    pub name: String,
}

impl super::runtime::SoraDecode for DropGroup {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for DropGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("DropGroup");
        builder.field("id", &self.id);
        builder.field("name", &self.name);
        builder.finish()
    }
}
