package showcase

type SkillEffect struct {
	Element ElementType
	Power   int32
	Radius  float32
}

func decodeSkillEffect(reader *SoraReader) (SkillEffect, error) {
	var value SkillEffect
	var err error
	value.Element, err = decodeElementType(reader)
	if err != nil {
		return value, err
	}
	value.Power, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Radius, err = reader.ReadFloat32()
	if err != nil {
		return value, err
	}
	return value, nil
}
