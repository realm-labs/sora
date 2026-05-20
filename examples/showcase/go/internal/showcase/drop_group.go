package showcase

type DropGroup struct {
	Id   int32
	Name string
}

func decodeDropGroup(reader *SoraReader) (DropGroup, error) {
	var value DropGroup
	var err error
	value.Id, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Name, err = reader.ReadString()
	if err != nil {
		return value, err
	}
	return value, nil
}
