use super::rarity::Rarity;
use super::vec3::Vec3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Character {
    pub id: i32,
    pub name: String,
    pub rarity: Rarity,
    pub base_level: i32,
    pub base_skill: i32,
    pub starter_items: Vec<i32>,
    pub spawn_pos: Vec3,
}

impl super::runtime::SoraDecode for Character {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
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

impl std::fmt::Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Character");
        builder.field("id", &self.id);
        builder.field("name", &self.name);
        builder.field("rarity", &self.rarity);
        builder.field("base_level", &self.base_level);
        builder.field("base_skill", &self.base_skill);
        builder.field("starter_items", &self.starter_items);
        builder.field("spawn_pos", &self.spawn_pos);
        builder.finish()
    }
}
