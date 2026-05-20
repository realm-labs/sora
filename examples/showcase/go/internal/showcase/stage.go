package showcase

type Stage struct {
    Id int32
    Name string
    MonsterIds []int32
    RecommendedPower int32
    FirstClearRewards []Reward
}

func decodeStage(reader *SoraReader) (Stage, error) {
    var value Stage
    var err error
    value.Id, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.Name, err = reader.ReadString()
    if err != nil {
        return value, err
    }
    value.MonsterIds, err = ReadList(reader, func(reader *SoraReader) (int32, error) { return reader.ReadInt32() })
    if err != nil {
        return value, err
    }
    value.RecommendedPower, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.FirstClearRewards, err = ReadList(reader, func(reader *SoraReader) (Reward, error) { return decodeReward(reader) })
    if err != nil {
        return value, err
    }
    return value, nil
}
