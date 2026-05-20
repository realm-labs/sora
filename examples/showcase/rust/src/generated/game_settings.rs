use super::vec3::Vec3;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameSettings {
    pub version: String,
    pub daily_reset_hour: i32,
    pub starting_gold: i32,
    pub spawn_pos: Vec3,
    pub starter_items: Vec<i32>,
}

impl super::runtime::SoraDecode for GameSettings {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            version: <String as super::runtime::SoraDecode>::decode(reader)?,
            daily_reset_hour: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            starting_gold: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            spawn_pos: <Vec3 as super::runtime::SoraDecode>::decode(reader)?,
            starter_items: <Vec<i32> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for GameSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("GameSettings");
        builder.field("version", &self.version);
        builder.field("daily_reset_hour", &self.daily_reset_hour);
        builder.field("starting_gold", &self.starting_gold);
        builder.field("spawn_pos", &self.spawn_pos);
        builder.field("starter_items", &self.starter_items);
        builder.finish()
    }
}
