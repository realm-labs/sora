package game_config_showcase

data class GachaItem(
    val poolId: Int,
    val itemId: Int,
    val rarity: Rarity,
    val weight: Float,
) {
    companion object {
        fun decode(reader: SoraReader): GachaItem =
            GachaItem(
                poolId = reader.readI32(),
                itemId = reader.readI32(),
                rarity = Rarity.decode(reader),
                weight = reader.readF32(),
            )
    }
}