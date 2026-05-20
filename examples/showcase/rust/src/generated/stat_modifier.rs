
use super::stat_type::StatType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatModifier {
    #[serde(rename = "stat")]
    pub stat: StatType,
    #[serde(rename = "value")]
    pub value: f32,
    #[serde(rename = "is_percent")]
    pub is_percent: bool,
}

impl super::runtime::SoraDecode for StatModifier {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            stat: <StatType as super::runtime::SoraDecode>::decode(reader)?,
            value: <f32 as super::runtime::SoraDecode>::decode(reader)?,
            is_percent: <bool as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}