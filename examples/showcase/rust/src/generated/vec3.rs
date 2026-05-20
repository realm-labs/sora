#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl super::runtime::SoraDecode for Vec3 {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            x: <f32 as super::runtime::SoraDecode>::decode(reader)?,
            y: <f32 as super::runtime::SoraDecode>::decode(reader)?,
            z: <f32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
