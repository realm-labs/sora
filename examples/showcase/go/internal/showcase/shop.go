package showcase

type Shop struct {
    Id int32
    Name string
    Currency ResourceKind
}

func decodeShop(reader *SoraReader) (Shop, error) {
    var value Shop
    var err error
    value.Id, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.Name, err = reader.ReadString()
    if err != nil {
        return value, err
    }
    value.Currency, err = decodeResourceKind(reader)
    if err != nil {
        return value, err
    }
    return value, nil
}
