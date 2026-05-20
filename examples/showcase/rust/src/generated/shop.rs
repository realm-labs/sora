use super::resource_kind::ResourceKind;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Shop {
    pub id: i32,
    pub name: String,
    pub currency: ResourceKind,
}

impl super::runtime::SoraDecode for Shop {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            currency: <ResourceKind as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
