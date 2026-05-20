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
