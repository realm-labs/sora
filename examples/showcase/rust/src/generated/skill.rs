
use super::element_type::ElementType;
use super::resource_cost::ResourceCost;
use super::skill_effect::SkillEffect;
use super::vec3::Vec3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "element")]
    pub element: ElementType,
    /// Tuple cost, e.g. Gold,0,150
    #[serde(rename = "cost")]
    pub cost: ResourceCost,
    /// JSON object with element/power/radius
    #[serde(rename = "effect")]
    pub effect: SkillEffect,
    #[serde(rename = "required_level")]
    pub required_level: i32,
    /// Optional item requirement
    #[serde(rename = "required_item")]
    pub required_item: Option<i32>,
    #[serde(rename = "cast_origin")]
    pub cast_origin: Vec3,
}

impl super::runtime::SoraDecode for Skill {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
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