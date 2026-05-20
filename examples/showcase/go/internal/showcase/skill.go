package showcase

type Skill struct {
	Id            int32
	Name          string
	Element       ElementType
	Cost          ResourceCost
	Effect        SkillEffect
	RequiredLevel int32
	RequiredItem  *int32
	CastOrigin    Vec3
}

func decodeSkill(reader *SoraReader) (Skill, error) {
	var value Skill
	var err error
	value.Id, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Name, err = reader.ReadString()
	if err != nil {
		return value, err
	}
	value.Element, err = decodeElementType(reader)
	if err != nil {
		return value, err
	}
	value.Cost, err = decodeResourceCost(reader)
	if err != nil {
		return value, err
	}
	value.Effect, err = decodeSkillEffect(reader)
	if err != nil {
		return value, err
	}
	value.RequiredLevel, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.RequiredItem, err = ReadOptional(reader, func(reader *SoraReader) (int32, error) { return reader.ReadInt32() })
	if err != nil {
		return value, err
	}
	value.CastOrigin, err = decodeVec3(reader)
	if err != nil {
		return value, err
	}
	return value, nil
}
