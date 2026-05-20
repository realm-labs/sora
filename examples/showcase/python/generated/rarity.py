from __future__ import annotations

from enum import Enum

from .sora_runtime import SoraReadError, SoraReader


class Rarity(Enum):
    COMMON = "Common"
    UNCOMMON = "Uncommon"
    RARE = "Rare"
    EPIC = "Epic"
    LEGENDARY = "Legendary"

    @staticmethod
    def decode(reader: SoraReader) -> Rarity:
        ordinal = reader.read_u32()
        if ordinal == 0:
            return Rarity.COMMON
        if ordinal == 1:
            return Rarity.UNCOMMON
        if ordinal == 2:
            return Rarity.RARE
        if ordinal == 3:
            return Rarity.EPIC
        if ordinal == 4:
            return Rarity.LEGENDARY
        raise SoraReadError(f"invalid enum ordinal {ordinal} for Rarity")
