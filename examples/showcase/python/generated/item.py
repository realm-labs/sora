from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    from .item_type import ItemType
    from .resource_cost import ResourceCost


@dataclass(frozen=True, slots=True)
class Item:
    # Item id
    id: int
    # Display name
    name: str
    # Item category
    item_type: ItemType
    # Stack limit; blank cells use the default
    max_stack: int
    # Tuple: kind,id,count
    price: ResourceCost
    # JSON string array
    tags: list[str]

    @staticmethod
    def decode(reader: SoraReader) -> Item:
        from .item_type import ItemType
        from .resource_cost import ResourceCost
        id = reader.read_i32()
        name = reader.read_string()
        item_type = ItemType.decode(reader)
        max_stack = reader.read_i32()
        price = ResourceCost.decode(reader)
        tags = reader.read_list(lambda: reader.read_string())
        return Item(
            id=id,
            name=name,
            item_type=item_type,
            max_stack=max_stack,
            price=price,
            tags=tags,
        )
