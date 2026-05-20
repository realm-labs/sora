use super::stat_type::StatType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatModifier {
    pub stat: StatType,
    pub value: f32,
    pub is_percent: bool,
}

impl super::runtime::SoraDecode for StatModifier {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            stat: <StatType as super::runtime::SoraDecode>::decode(reader)?,
            value: <f32 as super::runtime::SoraDecode>::decode(reader)?,
            is_percent: <bool as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for StatModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("StatModifier");
        builder.field("stat", &self.stat);
        builder.field("value", &self.value);
        builder.field("is_percent", &self.is_percent);
        builder.finish()
    }
}
