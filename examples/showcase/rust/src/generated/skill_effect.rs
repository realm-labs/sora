use super::element_type::ElementType;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SkillEffect {
    pub element: ElementType,
    pub power: i32,
    pub radius: f32,
}

impl super::runtime::SoraDecode for SkillEffect {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            element: <ElementType as super::runtime::SoraDecode>::decode(reader)?,
            power: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            radius: <f32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
