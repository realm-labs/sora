package showcase

import "fmt"

type RewardAction interface {
    isRewardAction()
}
type RewardActionAddItem struct {
    ItemId int32
    Count int32
}

func (RewardActionAddItem) isRewardAction() {}
type RewardActionAddBuff struct {
    BuffId int32
    Duration float32
}

func (RewardActionAddBuff) isRewardAction() {}
type RewardActionUnlockStage struct {
    StageId int32
}

func (RewardActionUnlockStage) isRewardAction() {}
type RewardActionSendMail struct {
    MailId int32
}

func (RewardActionSendMail) isRewardAction() {}
func decodeRewardAction(reader *SoraReader) (RewardAction, error) {
    ordinal, err := reader.ReadUInt32()
    if err != nil {
        return nil, err
    }
    switch ordinal {
    case 0:
        var value RewardActionAddItem
        value.ItemId, err = reader.ReadInt32()
        if err != nil {
            return nil, err
        }
        value.Count, err = reader.ReadInt32()
        if err != nil {
            return nil, err
        }
        return value, nil
    case 1:
        var value RewardActionAddBuff
        value.BuffId, err = reader.ReadInt32()
        if err != nil {
            return nil, err
        }
        value.Duration, err = reader.ReadFloat32()
        if err != nil {
            return nil, err
        }
        return value, nil
    case 2:
        var value RewardActionUnlockStage
        value.StageId, err = reader.ReadInt32()
        if err != nil {
            return nil, err
        }
        return value, nil
    case 3:
        var value RewardActionSendMail
        value.MailId, err = reader.ReadInt32()
        if err != nil {
            return nil, err
        }
        return value, nil
    default:
        return nil, fmt.Errorf("invalid union ordinal %d for RewardAction", ordinal)
    }
}
