from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    from .resource_cost import ResourceCost


@dataclass(frozen=True, slots=True)
class Achievement:
    id: int
    title_key: str
    target_count: int
    reward: ResourceCost

    @staticmethod
    def decode(reader: SoraReader) -> Achievement:
        from .resource_cost import ResourceCost
        id = reader.read_i32()
        title_key = reader.read_string()
        target_count = reader.read_i64()
        reward = ResourceCost.decode(reader)
        return Achievement(
            id=id,
            title_key=title_key,
            target_count=target_count,
            reward=reward,
        )
