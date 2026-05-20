from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from .sora_runtime import SoraReader


if TYPE_CHECKING:
    from .resource_kind import ResourceKind


@dataclass(frozen=True, slots=True)
class ResourceCost:
    kind: ResourceKind
    id: int
    count: int

    @staticmethod
    def decode(reader: SoraReader) -> ResourceCost:
        from .resource_kind import ResourceKind
        kind = ResourceKind.decode(reader)
        id = reader.read_i32()
        count = reader.read_i32()
        return ResourceCost(
            kind=kind,
            id=id,
            count=count,
        )
