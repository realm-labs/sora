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
