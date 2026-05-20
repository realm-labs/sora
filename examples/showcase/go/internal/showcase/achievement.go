package showcase

type Achievement struct {
	Id          int32
	TitleKey    string
	TargetCount int64
	Reward      ResourceCost
}

func decodeAchievement(reader *SoraReader) (Achievement, error) {
	var value Achievement
	var err error
	value.Id, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.TitleKey, err = reader.ReadString()
	if err != nil {
		return value, err
	}
	value.TargetCount, err = reader.ReadInt64()
	if err != nil {
		return value, err
	}
	value.Reward, err = decodeResourceCost(reader)
	if err != nil {
		return value, err
	}
	return value, nil
}
