from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    from .resource_cost import ResourceCost


@dataclass(frozen=True, slots=True)
class Dungeon:
    id: int
    name: str
    stage_ids: list[int]
    entry_cost: ResourceCost

    @staticmethod
    def decode(reader: SoraReader) -> Dungeon:
        from .resource_cost import ResourceCost
        id = reader.read_i32()
        name = reader.read_string()
        stage_ids = reader.read_list(lambda: reader.read_i32())
        entry_cost = ResourceCost.decode(reader)
        return Dungeon(
            id=id,
            name=name,
            stage_ids=stage_ids,
            entry_cost=entry_cost,
        )
