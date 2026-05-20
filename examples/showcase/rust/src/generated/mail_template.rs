
use super::mail_type::MailType;
use super::reward::Reward;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MailTemplate {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "mail_type")]
    pub mail_type: MailType,
    #[serde(rename = "title_key")]
    pub title_key: String,
    #[serde(rename = "body_key")]
    pub body_key: String,
    #[serde(rename = "rewards")]
    pub rewards: Vec<Reward>,
}

impl super::runtime::SoraDecode for MailTemplate {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            mail_type: <MailType as super::runtime::SoraDecode>::decode(reader)?,
            title_key: <String as super::runtime::SoraDecode>::decode(reader)?,
            body_key: <String as super::runtime::SoraDecode>::decode(reader)?,
            rewards: <Vec<Reward> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}