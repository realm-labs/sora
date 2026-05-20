use super::reward::Reward;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stage {
    pub id: i32,
    pub name: String,
    pub monster_ids: Vec<i32>,
    pub recommended_power: i32,
    pub first_clear_rewards: Vec<Reward>,
}

impl super::runtime::SoraDecode for Stage {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            monster_ids: <Vec<i32> as super::runtime::SoraDecode>::decode(reader)?,
            recommended_power: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            first_clear_rewards: <Vec<Reward> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Stage");
        builder.field("id", &self.id);
        builder.field("name", &self.name);
        builder.field("monster_ids", &self.monster_ids);
        builder.field("recommended_power", &self.recommended_power);
        builder.field("first_clear_rewards", &self.first_clear_rewards);
        builder.finish()
    }
}
