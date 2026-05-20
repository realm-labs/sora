from __future__ import annotations

from enum import Enum

from .sora_runtime import SoraReadError, SoraReader


class MailType(Enum):
    SYSTEM = "System"
    EVENT = "Event"
    COMPENSATION = "Compensation"

    @staticmethod
    def decode(reader: SoraReader) -> MailType:
        ordinal = reader.read_u32()
        if ordinal == 0:
            return MailType.SYSTEM
        if ordinal == 1:
            return MailType.EVENT
        if ordinal == 2:
            return MailType.COMPENSATION
        raise SoraReadError(f"invalid enum ordinal {ordinal} for MailType")
