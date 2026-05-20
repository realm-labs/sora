from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    pass


@dataclass(frozen=True, slots=True)
class Dialogue:
    id: int
    speaker_key: str
    lines: list[str]

    @staticmethod
    def decode(reader: SoraReader) -> Dialogue:
        id = reader.read_i32()
        speaker_key = reader.read_string()
        lines = reader.read_list(lambda: reader.read_string())
        return Dialogue(
            id=id,
            speaker_key=speaker_key,
            lines=lines,
        )
