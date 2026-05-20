package showcase

type Character struct {
	Id           int32
	Name         string
	Rarity       Rarity
	BaseLevel    int32
	BaseSkill    int32
	StarterItems []int32
	SpawnPos     Vec3
}

func decodeCharacter(reader *SoraReader) (Character, error) {
	var value Character
	var err error
	value.Id, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Name, err = reader.ReadString()
	if err != nil {
		return value, err
	}
	value.Rarity, err = decodeRarity(reader)
	if err != nil {
		return value, err
	}
	value.BaseLevel, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.BaseSkill, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.StarterItems, err = ReadList(reader, func(reader *SoraReader) (int32, error) { return reader.ReadInt32() })
	if err != nil {
		return value, err
	}
	value.SpawnPos, err = decodeVec3(reader)
	if err != nil {
		return value, err
	}
	return value, nil
}
