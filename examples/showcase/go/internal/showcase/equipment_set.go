package showcase

type EquipmentSet struct {
	Id          int32
	Name        string
	ItemIds     []int32
	BonusEffect SkillEffect
}

func decodeEquipmentSet(reader *SoraReader) (EquipmentSet, error) {
	var value EquipmentSet
	var err error
	value.Id, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Name, err = reader.ReadString()
	if err != nil {
		return value, err
	}
	value.ItemIds, err = ReadList(reader, func(reader *SoraReader) (int32, error) { return reader.ReadInt32() })
	if err != nil {
		return value, err
	}
	value.BonusEffect, err = decodeSkillEffect(reader)
	if err != nil {
		return value, err
	}
	return value, nil
}
