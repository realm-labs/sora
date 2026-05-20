package showcase

type LevelExp struct {
    Level int32
    Exp int64
    UnlockFeature *string
}

func decodeLevelExp(reader *SoraReader) (LevelExp, error) {
    var value LevelExp
    var err error
    value.Level, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.Exp, err = reader.ReadInt64()
    if err != nil {
        return value, err
    }
    value.UnlockFeature, err = ReadOptional(reader, func(reader *SoraReader) (string, error) { return reader.ReadString() })
    if err != nil {
        return value, err
    }
    return value, nil
}
