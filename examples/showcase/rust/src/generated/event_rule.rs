
use super::event_condition::EventCondition;
use super::reward_action::RewardAction;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventRule {
    #[serde(rename = "id")]
    pub id: i32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "condition")]
    pub condition: EventCondition,
    #[serde(rename = "actions")]
    pub actions: Vec<RewardAction>,
}

impl super::runtime::SoraDecode for EventRule {
    fn decode(reader: &mut super::runtime::SoraReader<'_>) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            condition: <EventCondition as super::runtime::SoraDecode>::decode(reader)?,
            actions: <Vec<RewardAction> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
