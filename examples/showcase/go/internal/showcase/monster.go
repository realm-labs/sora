package showcase

type Monster struct {
	Id        int32
	Name      string
	Level     int32
	Element   ElementType
	DropGroup int32
	SpawnPos  Vec3
}

func decodeMonster(reader *SoraReader) (Monster, error) {
	var value Monster
	var err error
	value.Id, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Name, err = reader.ReadString()
	if err != nil {
		return value, err
	}
	value.Level, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Element, err = decodeElementType(reader)
	if err != nil {
		return value, err
	}
	value.DropGroup, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.SpawnPos, err = decodeVec3(reader)
	if err != nil {
		return value, err
	}
	return value, nil
}
