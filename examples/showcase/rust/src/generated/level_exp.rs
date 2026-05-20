#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LevelExp {
    pub level: i32,
    pub exp: i64,
    pub unlock_feature: Option<String>,
}

impl super::runtime::SoraDecode for LevelExp {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            level: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            exp: <i64 as super::runtime::SoraDecode>::decode(reader)?,
            unlock_feature: <Option<String> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for LevelExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("LevelExp");
        builder.field("level", &self.level);
        builder.field("exp", &self.exp);
        builder.field("unlock_feature", &self.unlock_feature);
        builder.finish()
    }
}
