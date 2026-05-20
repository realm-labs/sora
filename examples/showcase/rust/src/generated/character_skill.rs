#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CharacterSkill {
    pub character_id: i32,
    pub skill_id: i32,
    pub unlock_level: i32,
}

impl super::runtime::SoraDecode for CharacterSkill {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            character_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            skill_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            unlock_level: <i32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for CharacterSkill {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("CharacterSkill");
        builder.field("character_id", &self.character_id);
        builder.field("skill_id", &self.skill_id);
        builder.field("unlock_level", &self.unlock_level);
        builder.finish()
    }
}
