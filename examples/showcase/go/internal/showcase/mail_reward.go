package showcase

type MailReward struct {
	MailId int32
	Seq    int32
	ItemId int32
	Count  int32
}

func decodeMailReward(reader *SoraReader) (MailReward, error) {
	var value MailReward
	var err error
	value.MailId, err = reader.ReadInt32()
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
