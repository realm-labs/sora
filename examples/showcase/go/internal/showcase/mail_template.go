package showcase

type MailTemplate struct {
	Id       int32
	MailType MailType
	TitleKey string
	BodyKey  string
	Rewards  []Reward
}

func decodeMailTemplate(reader *SoraReader) (MailTemplate, error) {
	var value MailTemplate
	var err error
	value.Id, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.MailType, err = decodeMailType(reader)
	if err != nil {
		return value, err
	}
	value.TitleKey, err = reader.ReadString()
	if err != nil {
		return value, err
	}
	value.BodyKey, err = reader.ReadString()
	if err != nil {
		return value, err
	}
	value.Rewards, err = ReadList(reader, func(reader *SoraReader) (Reward, error) { return decodeReward(reader) })
	if err != nil {
		return value, err
	}
	return value, nil
}
