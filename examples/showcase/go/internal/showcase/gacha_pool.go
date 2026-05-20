package showcase

type GachaPool struct {
    Id int32
    Name string
    Cost ResourceCost
}

func decodeGachaPool(reader *SoraReader) (GachaPool, error) {
    var value GachaPool
    var err error
    value.Id, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.Name, err = reader.ReadString()
    if err != nil {
        return value, err
    }
    value.Cost, err = decodeResourceCost(reader)
    if err != nil {
        return value, err
    }
    return value, nil
}
