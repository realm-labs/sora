#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dialogue {
    pub id: i32,
    pub speaker_key: String,
    pub lines: Vec<String>,
}

impl super::runtime::SoraDecode for Dialogue {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            speaker_key: <String as super::runtime::SoraDecode>::decode(reader)?,
            lines: <Vec<String> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for Dialogue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Dialogue");
        builder.field("id", &self.id);
        builder.field("speaker_key", &self.speaker_key);
        builder.field("lines", &self.lines);
        builder.finish()
    }
}
