use super::element_type::ElementType;
use super::vec3::Vec3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Monster {
    pub id: i32,
    pub name: String,
    pub level: i32,
    pub element: ElementType,
    pub drop_group: i32,
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

impl std::fmt::Display for Monster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Monster");
        builder.field("id", &self.id);
        builder.field("name", &self.name);
        builder.field("level", &self.level);
        builder.field("element", &self.element);
        builder.field("drop_group", &self.drop_group);
        builder.field("spawn_pos", &self.spawn_pos);
        builder.finish()
    }
}
