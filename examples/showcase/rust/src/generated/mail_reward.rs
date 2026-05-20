

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailReward {
    #[serde(rename = "mail_id")]
    pub mail_id: i32,
    #[serde(rename = "seq")]
    pub seq: i32,
    #[serde(rename = "item_id")]
    pub item_id: i32,
    #[serde(rename = "count")]
    pub count: i32,
}

impl super::runtime::SoraDecode for MailReward {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            mail_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            seq: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}