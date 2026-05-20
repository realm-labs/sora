from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    pass


@dataclass(frozen=True, slots=True)
class Vec3:
    x: float
    y: float
    z: float

    @staticmethod
    def decode(reader: SoraReader) -> Vec3:
        x = reader.read_f32()
        y = reader.read_f32()
        z = reader.read_f32()
        return Vec3(
            x=x,
            y=y,
            z=z,
        )
