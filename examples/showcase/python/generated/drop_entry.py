from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    pass


@dataclass(frozen=True, slots=True)
class DropEntry:
    group_id: int
    seq: int
    item_id: int
    count: int
    weight: float

    @staticmethod
    def decode(reader: SoraReader) -> DropEntry:
        group_id = reader.read_i32()
        seq = reader.read_i32()
        item_id = reader.read_i32()
        count = reader.read_i32()
        weight = reader.read_f32()
        return DropEntry(
            group_id=group_id,
            seq=seq,
            item_id=item_id,
            count=count,
            weight=weight,
        )
