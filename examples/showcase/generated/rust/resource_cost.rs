use super::resource_kind::ResourceKind;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceCost {
    pub kind: ResourceKind,
    pub id: i32,
    pub count: i32,
}

impl super::runtime::SoraDecode for ResourceCost {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            kind: <ResourceKind as super::runtime::SoraDecode>::decode(reader)?,
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
