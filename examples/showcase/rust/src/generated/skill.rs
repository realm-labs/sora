use super::element_type::ElementType;
use super::resource_cost::ResourceCost;
use super::skill_effect::SkillEffect;
use super::vec3::Vec3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    pub id: i32,
    pub name: String,
    pub element: ElementType,
    /// Tuple cost, e.g. Gold,0,150
    pub cost: ResourceCost,
    /// JSON object with element/power/radius
    pub effect: SkillEffect,
    pub required_level: i32,
    /// Optional item requirement
    pub required_item: Option<i32>,
    pub cast_origin: Vec3,
}

impl super::runtime::SoraDecode for Skill {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            element: <ElementType as super::runtime::SoraDecode>::decode(reader)?,
            cost: <ResourceCost as super::runtime::SoraDecode>::decode(reader)?,
            effect: <SkillEffect as super::runtime::SoraDecode>::decode(reader)?,
            required_level: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            required_item: <Option<i32> as super::runtime::SoraDecode>::decode(reader)?,
            cast_origin: <Vec3 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for Skill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Skill");
        builder.field("id", &self.id);
        builder.field("name", &self.name);
        builder.field("element", &self.element);
        builder.field("cost", &self.cost);
        builder.field("effect", &self.effect);
        builder.field("required_level", &self.required_level);
        builder.field("required_item", &self.required_item);
        builder.field("cast_origin", &self.cast_origin);
        builder.finish()
    }
}
