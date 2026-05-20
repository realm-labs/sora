package showcase

type EventRule struct {
    Id int32
    Name string
    Condition EventCondition
    Actions []RewardAction
}

func decodeEventRule(reader *SoraReader) (EventRule, error) {
    var value EventRule
    var err error
    value.Id, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.Name, err = reader.ReadString()
    if err != nil {
        return value, err
    }
    value.Condition, err = decodeEventCondition(reader)
    if err != nil {
        return value, err
    }
    value.Actions, err = ReadList(reader, func(reader *SoraReader) (RewardAction, error) { return decodeRewardAction(reader) })
    if err != nil {
        return value, err
    }
    return value, nil
}
