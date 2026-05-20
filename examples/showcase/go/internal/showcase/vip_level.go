package showcase

type VipLevel struct {
	Level int32
	Cost  ResourceCost
	Perks []string
}

func decodeVipLevel(reader *SoraReader) (VipLevel, error) {
	var value VipLevel
	var err error
	value.Level, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Cost, err = decodeResourceCost(reader)
	if err != nil {
		return value, err
	}
	value.Perks, err = ReadList(reader, func(reader *SoraReader) (string, error) { return reader.ReadString() })
	if err != nil {
		return value, err
	}
	return value, nil
}
