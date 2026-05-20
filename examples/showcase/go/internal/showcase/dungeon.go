package showcase

type Dungeon struct {
    Id int32
    Name string
    StageIds []int32
    EntryCost ResourceCost
}

func decodeDungeon(reader *SoraReader) (Dungeon, error) {
    var value Dungeon
    var err error
    value.Id, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.Name, err = reader.ReadString()
    if err != nil {
        return value, err
    }
    value.StageIds, err = ReadList(reader, func(reader *SoraReader) (int32, error) { return reader.ReadInt32() })
    if err != nil {
        return value, err
    }
    value.EntryCost, err = decodeResourceCost(reader)
    if err != nil {
        return value, err
    }
    return value, nil
}
