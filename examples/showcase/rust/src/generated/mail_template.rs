use super::mail_type::MailType;
use super::reward::Reward;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailTemplate {
    pub id: i32,
    pub mail_type: MailType,
    pub title_key: String,
    pub body_key: String,
    pub rewards: Vec<Reward>,
}

impl super::runtime::SoraDecode for MailTemplate {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            mail_type: <MailType as super::runtime::SoraDecode>::decode(reader)?,
            title_key: <String as super::runtime::SoraDecode>::decode(reader)?,
            body_key: <String as super::runtime::SoraDecode>::decode(reader)?,
            rewards: <Vec<Reward> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
