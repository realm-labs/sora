use super::reward::Reward;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stage {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "monster_ids")]
    pub monster_ids: Vec<i32>,
    #[serde(rename = "recommended_power")]
    pub recommended_power: i32,
    #[serde(rename = "first_clear_rewards")]
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
