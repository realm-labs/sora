from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    from .resource_cost import ResourceCost


@dataclass(frozen=True, slots=True)
class Recipe:
    id: int
    result_item: int
    materials: list[ResourceCost]

    @staticmethod
    def decode(reader: SoraReader) -> Recipe:
        from .resource_cost import ResourceCost
        id = reader.read_i32()
        result_item = reader.read_i32()
        materials = reader.read_list(lambda: ResourceCost.decode(reader))
        return Recipe(
            id=id,
            result_item=result_item,
            materials=materials,
        )
