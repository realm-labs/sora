package game_config_showcase

data class EquipmentSet(
    val id: Int,
    val name: String,
    val itemIds: List<Int>,
    val bonusEffect: SkillEffect,
) {
    companion object {
        fun decode(reader: SoraReader): EquipmentSet =
            EquipmentSet(
                id = reader.readI32(),
                name = reader.readString(),
                itemIds = reader.readList { reader.readI32() },
                bonusEffect = SkillEffect.decode(reader),
            )
    }
}