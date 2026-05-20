package showcase

type ResourceCost struct {
	Kind  ResourceKind
	Id    int32
	Count int32
}

func decodeResourceCost(reader *SoraReader) (ResourceCost, error) {
	var value ResourceCost
	var err error
	value.Kind, err = decodeResourceKind(reader)
	if err != nil {
		return value, err
	}
	value.Id, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	value.Count, err = reader.ReadInt32()
	if err != nil {
		return value, err
	}
	return value, nil
}
