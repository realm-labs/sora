from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    from .vec3 import Vec3


@dataclass(frozen=True, slots=True)
class GameSettings:
    version: str
    daily_reset_hour: int
    starting_gold: int
    spawn_pos: Vec3
    starter_items: list[int]

    @staticmethod
    def decode(reader: SoraReader) -> GameSettings:
        from .vec3 import Vec3
        version = reader.read_string()
        daily_reset_hour = reader.read_i32()
        starting_gold = reader.read_i32()
        spawn_pos = Vec3.decode(reader)
        starter_items = reader.read_list(lambda: reader.read_i32())
        return GameSettings(
            version=version,
            daily_reset_hour=daily_reset_hour,
            starting_gold=starting_gold,
            spawn_pos=spawn_pos,
            starter_items=starter_items,
        )
