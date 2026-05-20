package game_config_showcase

data class Item(
    val id: Int,
    val name: String,
    val itemType: ItemType,
    val maxStack: Int,
    val price: ResourceCost,
    val tags: List<String>,
) {
    companion object {
        fun decode(reader: SoraReader): Item =
            Item(
                id = reader.readI32(),
                name = reader.readString(),
                itemType = ItemType.decode(reader),
                maxStack = reader.readI32(),
                price = ResourceCost.decode(reader),
                tags = reader.readList { reader.readString() },
            )
    }
}