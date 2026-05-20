
use super::stat_modifier::StatModifier;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Buff {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "duration")]
    pub duration: f32,
    #[serde(rename = "modifiers")]
    pub modifiers: Vec<StatModifier>,
}

impl super::runtime::SoraDecode for Buff {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            duration: <f32 as super::runtime::SoraDecode>::decode(reader)?,
            modifiers: <Vec<StatModifier> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
