use super::event_condition::EventCondition;
use super::reward_action::RewardAction;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventRule {
    pub id: i32,
    pub name: String,
    pub condition: EventCondition,
    pub actions: Vec<RewardAction>,
}

impl super::runtime::SoraDecode for EventRule {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        Ok(Self {
            id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            name: <String as super::runtime::SoraDecode>::decode(reader)?,
            condition: <EventCondition as super::runtime::SoraDecode>::decode(reader)?,
            actions: <Vec<RewardAction> as super::runtime::SoraDecode>::decode(reader)?,
        })
    }
}
