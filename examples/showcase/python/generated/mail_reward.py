from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    pass


@dataclass(frozen=True, slots=True)
class MailReward:
    mail_id: int
    seq: int
    item_id: int
    count: int

    @staticmethod
    def decode(reader: SoraReader) -> MailReward:
        mail_id = reader.read_i32()
        seq = reader.read_i32()
        item_id = reader.read_i32()
        count = reader.read_i32()
        return MailReward(
            mail_id=mail_id,
            seq=seq,
            item_id=item_id,
            count=count,
        )
