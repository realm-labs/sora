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
