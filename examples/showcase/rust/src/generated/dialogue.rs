

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dialogue {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "speaker_key")]
    pub speaker_key: String,
    #[serde(rename = "lines")]
    pub lines: Vec<String>,
}

impl super::runtime::SoraDecode for Dialogue {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            speaker_key: <String as super::runtime::SoraDecode>::decode(reader)?,
            lines: <Vec<String> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}