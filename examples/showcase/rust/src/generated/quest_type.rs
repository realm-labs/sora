#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum QuestType {
    Main,
    Side,
    Daily,
}

impl super::runtime::SoraDecode for QuestType {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        match reader.read_u32()? {
            0 => Ok(Self::Main),
            1 => Ok(Self::Side),
            2 => Ok(Self::Daily),
            value => Err(super::runtime::SoraReadError::new(format!(
                "invalid enum ordinal {} for QuestType",
                value
            ))),
        }
    }
}

impl std::fmt::Display for QuestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Main => f.write_str("Main"),
            Self::Side => f.write_str("Side"),
            Self::Daily => f.write_str("Daily"),
        }
    }
}
