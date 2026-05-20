package showcase

type Localization struct {
    Key string
    ZhCn string
    EnUs string
    Note *string
}

func decodeLocalization(reader *SoraReader) (Localization, error) {
    var value Localization
    var err error
    value.Key, err = reader.ReadString()
    if err != nil {
        return value, err
    }
    value.ZhCn, err = reader.ReadString()
    if err != nil {
        return value, err
    }
    value.EnUs, err = reader.ReadString()
    if err != nil {
        return value, err
    }
    value.Note, err = ReadOptional(reader, func(reader *SoraReader) (string, error) { return reader.ReadString() })
    if err != nil {
        return value, err
    }
    return value, nil
}
