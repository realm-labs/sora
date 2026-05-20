#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MailType {
    System,
    Event,
    Compensation,
}

impl super::runtime::SoraDecode for MailType {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        match reader.read_u32()? {
            0 => Ok(Self::System),
            1 => Ok(Self::Event),
            2 => Ok(Self::Compensation),
            value => Err(super::runtime::SoraReadError::new(format!(
                "invalid enum ordinal {} for MailType",
                value
            ))),
        }
    }
}

impl std::fmt::Display for MailType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => f.write_str("System"),
            Self::Event => f.write_str("Event"),
            Self::Compensation => f.write_str("Compensation"),
        }
    }
}
