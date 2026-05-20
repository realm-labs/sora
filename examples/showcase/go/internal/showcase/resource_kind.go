package showcase

import "fmt"

type ResourceKind int32

const (
	ResourceKindItem    ResourceKind = 0
	ResourceKindGold    ResourceKind = 1
	ResourceKindDiamond ResourceKind = 2
)

func decodeResourceKind(reader *SoraReader) (ResourceKind, error) {
	ordinal, err := reader.ReadUInt32()
	if err != nil {
		return 0, err
	}
	switch ordinal {
	case 0:
		return ResourceKindItem, nil
	case 1:
		return ResourceKindGold, nil
	case 2:
		return ResourceKindDiamond, nil
	default:
		return 0, fmt.Errorf("invalid enum ordinal %d for ResourceKind", ordinal)
	}
}
