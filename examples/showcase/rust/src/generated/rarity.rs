#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl super::runtime::SoraDecode for Rarity {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        match reader.read_u32()? {
            0 => Ok(Self::Common),
            1 => Ok(Self::Uncommon),
            2 => Ok(Self::Rare),
            3 => Ok(Self::Epic),
            4 => Ok(Self::Legendary),
            value => Err(super::runtime::SoraReadError::new(format!(
                "invalid enum ordinal {} for Rarity",
                value
            ))),
        }
    }
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Common => f.write_str("Common"),
            Self::Uncommon => f.write_str("Uncommon"),
            Self::Rare => f.write_str("Rare"),
            Self::Epic => f.write_str("Epic"),
            Self::Legendary => f.write_str("Legendary"),
        }
    }
}
