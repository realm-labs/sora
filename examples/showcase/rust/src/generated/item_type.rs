#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ItemType {
    Weapon,
    Armor,
    Currency,
    Material,
    Consumable,
}

impl super::runtime::SoraDecode for ItemType {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        match reader.read_u32()? {
            0 => Ok(Self::Weapon),
            1 => Ok(Self::Armor),
            2 => Ok(Self::Currency),
            3 => Ok(Self::Material),
            4 => Ok(Self::Consumable),
            value => Err(super::runtime::SoraReadError::new(format!(
                "invalid enum ordinal {} for ItemType",
                value
            ))),
        }
    }
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Weapon => f.write_str("Weapon"),
            Self::Armor => f.write_str("Armor"),
            Self::Currency => f.write_str("Currency"),
            Self::Material => f.write_str("Material"),
            Self::Consumable => f.write_str("Consumable"),
        }
    }
}
