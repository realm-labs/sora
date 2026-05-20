#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailReward {
    pub mail_id: i32,
    pub seq: i32,
    pub item_id: i32,
    pub count: i32,
}

impl super::runtime::SoraDecode for MailReward {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            mail_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            seq: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}

impl std::fmt::Display for MailReward {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("MailReward");
        builder.field("mail_id", &self.mail_id);
        builder.field("seq", &self.seq);
        builder.field("item_id", &self.item_id);
        builder.field("count", &self.count);
        builder.finish()
    }
}
