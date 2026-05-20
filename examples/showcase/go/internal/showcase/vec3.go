package showcase

type Vec3 struct {
	X float32
	Y float32
	Z float32
}

func decodeVec3(reader *SoraReader) (Vec3, error) {
	var value Vec3
	var err error
	value.X, err = reader.ReadFloat32()
	if err != nil {
		return value, err
	}
	value.Y, err = reader.ReadFloat32()
	if err != nil {
		return value, err
	}
	value.Z, err = reader.ReadFloat32()
	if err != nil {
		return value, err
	}
	return value, nil
}
