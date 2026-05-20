from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    from .stat_type import StatType


@dataclass(frozen=True, slots=True)
class StatModifier:
    stat: StatType
    value: float
    is_percent: bool

    @staticmethod
    def decode(reader: SoraReader) -> StatModifier:
        from .stat_type import StatType
        stat = StatType.decode(reader)
        value = reader.read_f32()
        is_percent = reader.read_bool()
        return StatModifier(
            stat=stat,
            value=value,
            is_percent=is_percent,
        )
