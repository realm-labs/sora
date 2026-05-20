package showcase

type Dialogue struct {
    Id int32
    SpeakerKey string
    Lines []string
}

func decodeDialogue(reader *SoraReader) (Dialogue, error) {
    var value Dialogue
    var err error
    value.Id, err = reader.ReadInt32()
    if err != nil {
        return value, err
    }
    value.SpeakerKey, err = reader.ReadString()
    if err != nil {
        return value, err
    }
    value.Lines, err = ReadList(reader, func(reader *SoraReader) (string, error) { return reader.ReadString() })
    if err != nil {
        return value, err
    }
    return value, nil
}
