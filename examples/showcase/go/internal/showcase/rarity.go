package showcase

import "fmt"

type Rarity int32

const (
	RarityCommon    Rarity = 0
	RarityUncommon  Rarity = 1
	RarityRare      Rarity = 2
	RarityEpic      Rarity = 3
	RarityLegendary Rarity = 4
)

func decodeRarity(reader *SoraReader) (Rarity, error) {
	ordinal, err := reader.ReadUInt32()
	if err != nil {
		return 0, err
	}
	switch ordinal {
	case 0:
		return RarityCommon, nil
	case 1:
		return RarityUncommon, nil
	case 2:
		return RarityRare, nil
	case 3:
		return RarityEpic, nil
	case 4:
		return RarityLegendary, nil
	default:
		return 0, fmt.Errorf("invalid enum ordinal %d for Rarity", ordinal)
	}
}
