#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Localization {
    pub key: String,
    pub zh_cn: String,
    pub en_us: String,
    pub note: Option<String>,
}

impl super::runtime::SoraDecode for Localization {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            key: <String as super::runtime::SoraDecode>::decode(reader)?,
            zh_cn: <String as super::runtime::SoraDecode>::decode(reader)?,
            en_us: <String as super::runtime::SoraDecode>::decode(reader)?,
            note: <Option<String> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for Localization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("Localization");
        builder.field("key", &self.key);
        builder.field("zh_cn", &self.zh_cn);
        builder.field("en_us", &self.en_us);
        builder.field("note", &self.note);
        builder.finish()
    }
}
