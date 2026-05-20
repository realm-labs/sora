#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ResourceKind {
    Item,
    Gold,
    Diamond,
}

impl super::runtime::SoraDecode for ResourceKind {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        match reader.read_u32()? {
            0 => Ok(Self::Item),
            1 => Ok(Self::Gold),
            2 => Ok(Self::Diamond),
            value => Err(super::runtime::SoraReadError::new(format!(
                "invalid enum ordinal {} for ResourceKind",
                value
            ))),
        }
    }
}

impl std::fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Item => f.write_str("Item"),
            Self::Gold => f.write_str("Gold"),
            Self::Diamond => f.write_str("Diamond"),
        }
    }
}
