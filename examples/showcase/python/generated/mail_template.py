from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    from .mail_type import MailType
    from .reward import Reward


@dataclass(frozen=True, slots=True)
class MailTemplate:
    id: int
    mail_type: MailType
    title_key: str
    body_key: str
    rewards: list[Reward]

    @staticmethod
    def decode(reader: SoraReader) -> MailTemplate:
        from .mail_type import MailType
        from .reward import Reward
        id = reader.read_i32()
        mail_type = MailType.decode(reader)
        title_key = reader.read_string()
        body_key = reader.read_string()
        rewards = reader.read_list(lambda: Reward.decode(reader))
        return MailTemplate(
            id=id,
            mail_type=mail_type,
            title_key=title_key,
            body_key=body_key,
            rewards=rewards,
        )
