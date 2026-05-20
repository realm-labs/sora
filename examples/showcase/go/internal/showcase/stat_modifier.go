package showcase

type StatModifier struct {
	Stat      StatType
	Value     float32
	IsPercent bool
}

func decodeStatModifier(reader *SoraReader) (StatModifier, error) {
	var value StatModifier
	var err error
	value.Stat, err = decodeStatType(reader)
	if err != nil {
		return value, err
	}
	value.Value, err = reader.ReadFloat32()
	if err != nil {
		return value, err
	}
	value.IsPercent, err = reader.ReadBool()
	if err != nil {
		return value, err
	}
	return value, nil
}
