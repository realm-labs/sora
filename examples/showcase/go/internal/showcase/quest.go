package showcase

type Quest struct {
    Id int32
    QuestType QuestType
    Title string
    RequiredItem int32
    UnlockSkills []int32
    StartPos Vec3
    Rewards []Reward
}

func decodeQuest(reader *SoraReader) (Quest, error) {
    var value Quest
    var err error
    value.Id, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.QuestType, err = decodeQuestType(reader)
    if err != nil {
        return value, err
    }
    value.Title, err = reader.ReadString()
    if err != nil {
        return value, err
    }
    value.RequiredItem, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.UnlockSkills, err = ReadList(reader, func(reader *SoraReader) (int32, error) { return reader.ReadInt32() })
    if err != nil {
        return value, err
    }
    value.StartPos, err = decodeVec3(reader)
    if err != nil {
        return value, err
    }
    value.Rewards, err = ReadList(reader, func(reader *SoraReader) (Reward, error) { return decodeReward(reader) })
    if err != nil {
        return value, err
    }
    return value, nil
}
