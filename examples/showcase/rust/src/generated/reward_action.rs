#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RewardAction {
    AddItem { item_id: i32, count: i32 },
    AddBuff { buff_id: i32, duration: f32 },
    UnlockStage { stage_id: i32 },
    SendMail { mail_id: i32 },
}

impl super::runtime::SoraDecode for RewardAction {
    fn decode(
        reader: &mut super::runtime::SoraReader<'_>,
    ) -> Result<Self, super::runtime::SoraReadError> {
        match reader.read_u32()? {
            0 => Ok(Self::AddItem {
                item_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
                count: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            }),
            1 => Ok(Self::AddBuff {
                buff_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
                duration: <f32 as super::runtime::SoraDecode>::decode(reader)?,
            }),
            2 => Ok(Self::UnlockStage {
                stage_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            }),
            3 => Ok(Self::SendMail {
                mail_id: <i32 as super::runtime::SoraDecode>::decode(reader)?,
            }),
            value => Err(super::runtime::SoraReadError::new(format!(
                "invalid union ordinal {} for RewardAction",
                value
            ))),
        }
    }
}
