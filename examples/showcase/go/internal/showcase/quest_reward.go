package showcase

type QuestReward struct {
	QuestId int32
	Seq     int32
	ItemId  int32
	Count   int32
}

func decodeQuestReward(reader *SoraReader) (QuestReward, error) {
	var value QuestReward
	var err error
	value.QuestId, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Seq, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.ItemId, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Count, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	return value, nil
}
