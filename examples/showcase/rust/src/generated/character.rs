
use super::rarity::Rarity;
use super::vec3::Vec3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Character {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "rarity")]
    pub rarity: Rarity,
    #[serde(rename = "base_level")]
    pub base_level: i32,
    #[serde(rename = "base_skill")]
    pub base_skill: i32,
    #[serde(rename = "starter_items")]
    pub starter_items: Vec<i32>,
    #[serde(rename = "spawn_pos")]
    pub spawn_pos: Vec3,
}

impl super::runtime::SoraDecode for Character {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            rarity: <Rarity as super::runtime::SoraDecode>::decode(reader)?,
            base_level: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            base_skill: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            starter_items: <Vec<i32> as super::runtime::SoraDecode>::decode(reader)?,
            spawn_pos: <Vec3 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
