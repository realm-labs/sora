package showcase

type GachaItem struct {
    PoolId int32
    ItemId int32
    Rarity Rarity
    Weight float32
}

func decodeGachaItem(reader *SoraReader) (GachaItem, error) {
    var value GachaItem
    var err error
    value.PoolId, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.ItemId, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.Rarity, err = decodeRarity(reader)
    if err != nil {
        return value, err
    }
    value.Weight, err = reader.ReadFloat32()
    if err != nil {
        return value, err
    }
    return value, nil
}
