from __future__ import annotations

from enum import Enum

from .sora_runtime import SoraReadError, SoraReader


class ResourceKind(Enum):
    ITEM = "Item"
    GOLD = "Gold"
    DIAMOND = "Diamond"

    @staticmethod
    def decode(reader: SoraReader) -> ResourceKind:
        ordinal = reader.read_u32()
        if ordinal == 0:
            return ResourceKind.ITEM
        if ordinal == 1:
            return ResourceKind.GOLD
        if ordinal == 2:
            return ResourceKind.DIAMOND
        raise SoraReadError(f"invalid enum ordinal {ordinal} for ResourceKind")
