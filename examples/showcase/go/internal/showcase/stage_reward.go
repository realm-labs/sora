package showcase

type StageReward struct {
    StageId int32
    Seq int32
    ItemId int32
    Count int32
}

func decodeStageReward(reader *SoraReader) (StageReward, error) {
    var value StageReward
    var err error
    value.StageId, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.Seq, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.ItemId, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.Count, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    return value, nil
}
