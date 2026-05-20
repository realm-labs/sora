#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ElementType {
    Fire,
    Ice,
    Lightning,
    Physical,
}

impl super::runtime::SoraDecode for ElementType {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        match reader.read_u32()? {
            0 => Ok(Self::Fire),
            1 => Ok(Self::Ice),
            2 => Ok(Self::Lightning),
            3 => Ok(Self::Physical),
            value => Err(super::runtime::SoraReadError::new(format!(
                "invalid enum ordinal {} for ElementType",
                value
            ))),
        }
    }
}

impl std::fmt::Display for ElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fire => f.write_str("Fire"),
            Self::Ice => f.write_str("Ice"),
            Self::Lightning => f.write_str("Lightning"),
            Self::Physical => f.write_str("Physical"),
        }
    }
}
