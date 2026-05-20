
use super::resource_kind::ResourceKind;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceCost {
    #[serde(rename = "kind")]
    pub kind: ResourceKind,
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "count")]
    pub count: i32,
}

impl super::runtime::SoraDecode for ResourceCost {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            kind: <ResourceKind as super::runtime::SoraDecode>::decode(reader)?,
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}