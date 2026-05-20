package showcase

type Item struct {
	Id       int32
	Name     string
	ItemType ItemType
	MaxStack int32
	Price    ResourceCost
	Tags     []string
}

func decodeItem(reader *SoraReader) (Item, error) {
	var value Item
	var err error
	value.Id, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Name, err = reader.ReadString()
	if err != nil {
		return value, err
	}
	value.ItemType, err = decodeItemType(reader)
	if err != nil {
		return value, err
	}
	value.MaxStack, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Price, err = decodeResourceCost(reader)
	if err != nil {
		return value, err
	}
	value.Tags, err = ReadList(reader, func(reader *SoraReader) (string, error) { return reader.ReadString() })
	if err != nil {
		return value, err
	}
	return value, nil
}
