#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StatType {
    Hp,
    Attack,
    Defense,
    Speed,
    CritRate,
}

impl super::runtime::SoraDecode for StatType {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        match reader.read_u32()? {
            0 => Ok(Self::Hp),
            1 => Ok(Self::Attack),
            2 => Ok(Self::Defense),
            3 => Ok(Self::Speed),
            4 => Ok(Self::CritRate),
            value => Err(super::runtime::SoraReadError::new(format!("invalid enum ordinal {} for StatType", value))),
        }
    }
}