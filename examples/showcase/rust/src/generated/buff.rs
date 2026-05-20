use super::stat_modifier::StatModifier;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Buff {
    pub id: i32,
    pub name: String,
    pub duration: f32,
    pub modifiers: Vec<StatModifier>,
}

impl super::runtime::SoraDecode for Buff {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            duration: <f32 as super::runtime::SoraDecode>::decode(reader)?,
            modifiers: <Vec<StatModifier> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for Buff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Buff");
        builder.field("id", &self.id);
        builder.field("name", &self.name);
        builder.field("duration", &self.duration);
        builder.field("modifiers", &self.modifiers);
        builder.finish()
    }
}
