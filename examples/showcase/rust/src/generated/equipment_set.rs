use super::skill_effect::SkillEffect;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EquipmentSet {
    pub id: i32,
    pub name: String,
    pub item_ids: Vec<i32>,
    pub bonus_effect: SkillEffect,
}

impl super::runtime::SoraDecode for EquipmentSet {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            item_ids: <Vec<i32> as super::runtime::SoraDecode>::decode(reader)?,
            bonus_effect: <SkillEffect as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for EquipmentSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("EquipmentSet");
        builder.field("id", &self.id);
        builder.field("name", &self.name);
        builder.field("item_ids", &self.item_ids);
        builder.field("bonus_effect", &self.bonus_effect);
        builder.finish()
    }
}
