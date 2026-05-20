use super::element_type::ElementType;
use super::vec3::Vec3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Monster {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "level")]
    pub level: i32,
    #[serde(rename = "element")]
    pub element: ElementType,
    #[serde(rename = "drop_group")]
    pub drop_group: i32,
    #[serde(rename = "spawn_pos")]
    pub spawn_pos: Vec3,
}

impl super::runtime::SoraDecode for Monster {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            level: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            element: <ElementType as super::runtime::SoraDecode>::decode(reader)?,
            drop_group: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            spawn_pos: <Vec3 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
