package game_config_showcase

data class VipLevel(
    val level: Int,
    val cost: ResourceCost,
    val perks: List<String>,
) {
    companion object {
        fun decode(reader: SoraReader): VipLevel =
            VipLevel(
                level = reader.readI32(),
                cost = ResourceCost.decode(reader),
                perks = reader.readList { reader.readString() },
            )
    }
}