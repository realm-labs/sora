from __future__ import annotations

import struct
from dataclasses import dataclass
from typing import Callable, TypeVar

SORA_BUNDLE_VERSION = 1
SORA_HEADER_LENGTH = 24
SECTION_KIND_MANIFEST = 0
SECTION_KIND_SCHEMA = 1
SECTION_KIND_TABLE = 2
COMPRESSION_NONE = 0

T = TypeVar("T")
K = TypeVar("K")
V = TypeVar("V")


class SoraReadError(Exception):
    pass


@dataclass(frozen=True, slots=True)
class SoraSection:
    kind: int
    compression: int
    name: str
    offset: int
    length: int
    uncompressed_length: int


class SoraBundle:
    def __init__(self, bytes_data: bytes, sections: list[SoraSection]) -> None:
        self._bytes = bytes_data
        self._sections = sections

    @staticmethod
    def parse(input_data: bytes | bytearray) -> SoraBundle:
        data = bytes(input_data)
        if len(data) < SORA_HEADER_LENGTH:
            raise SoraReadError("Sora bundle is shorter than header")

        if data[0:4] != b"SORA":
            raise SoraReadError("invalid Sora bundle magic")

        version = struct.unpack_from("<I", data, 4)[0]
        if version != SORA_BUNDLE_VERSION:
            raise SoraReadError(f"unsupported Sora bundle version {version}")

        header_length = struct.unpack_from("<I", data, 8)[0]
        directory_length = struct.unpack_from("<I", data, 12)[0]
        section_count = struct.unpack_from("<I", data, 16)[0]
        flags = struct.unpack_from("<I", data, 20)[0]

        if flags != 0:
            raise SoraReadError(f"unsupported Sora bundle flags {flags}")
        if header_length != SORA_HEADER_LENGTH or header_length > len(data):
            raise SoraReadError("invalid Sora bundle header length")

        directory_end = header_length + directory_length
        if directory_end > len(data):
            raise SoraReadError("Sora section directory exceeds bundle length")

        cursor = header_length
        sections: list[SoraSection] = []
        manifest_count = 0
        schema_count = 0
        table_names: set[str] = set()

        for _ in range(section_count):
            if cursor + 40 > directory_end:
                raise SoraReadError("truncated Sora section directory entry")

            kind, compression, name_length, entry_flags = struct.unpack_from(
                "<IIII",
                data,
                cursor,
            )
            offset = struct.unpack_from("<Q", data, cursor + 16)[0]
            length = struct.unpack_from("<Q", data, cursor + 24)[0]
            uncompressed_length = struct.unpack_from("<Q", data, cursor + 32)[0]

            if entry_flags != 0:
                raise SoraReadError(f"unsupported Sora section flags {entry_flags}")

            name_start = cursor + 40
            name_end = name_start + name_length
            if name_end > directory_end:
                raise SoraReadError("Sora section name exceeds directory")
            if offset + length > len(data):
                raise SoraReadError("Sora section payload exceeds bundle length")

            name = data[name_start:name_end].decode("utf-8")
            if kind == SECTION_KIND_MANIFEST:
                manifest_count += 1
                if name != "$manifest":
                    raise SoraReadError("Sora manifest section must be named `$manifest`")
            elif kind == SECTION_KIND_SCHEMA:
                schema_count += 1
                if name != "$schema":
                    raise SoraReadError("Sora schema section must be named `$schema`")
            elif kind == SECTION_KIND_TABLE:
                if name in table_names:
                    raise SoraReadError(f"duplicate Sora table section `{name}`")
                table_names.add(name)
            else:
                raise SoraReadError(f"unknown Sora section kind {kind}")

            if compression == COMPRESSION_NONE and uncompressed_length != length:
                raise SoraReadError("uncompressed Sora section length mismatch")

            sections.append(
                SoraSection(
                    kind=kind,
                    compression=compression,
                    name=name,
                    offset=offset,
                    length=length,
                    uncompressed_length=uncompressed_length,
                )
            )
            cursor = name_end

        if cursor != directory_end:
            raise SoraReadError("Sora section directory has trailing bytes")
        if manifest_count != 1:
            raise SoraReadError(
                f"expected exactly 1 Sora manifest section, got {manifest_count}"
            )
        if schema_count != 1:
            raise SoraReadError(
                f"expected exactly 1 Sora schema section, got {schema_count}"
            )

        return SoraBundle(data, sections)

    def decode_table(self, name: str, decode_fn: Callable[[SoraReader], T]) -> list[T]:
        section = next(
            (
                section
                for section in self._sections
                if section.kind == SECTION_KIND_TABLE and section.name == name
            ),
            None,
        )
        if section is None:
            raise SoraReadError(f"missing Sora table section `{name}`")
        if section.compression != COMPRESSION_NONE:
            raise SoraReadError(
                f"unsupported compression {section.compression} for table `{name}`"
            )
        if section.uncompressed_length != section.length:
            raise SoraReadError(f"table `{name}` has invalid uncompressed length")

        payload = self._bytes[section.offset : section.offset + section.length]
        return decode_rows(payload, decode_fn)


class SoraReader:
    def __init__(self, data: bytes) -> None:
        self._data = data
        self._cursor = 0

    def is_finished(self) -> bool:
        return self._cursor == len(self._data)

    def read_u8(self) -> int:
        if self._cursor >= len(self._data):
            raise SoraReadError("Sora reader reached end of row")
        value = self._data[self._cursor]
        self._cursor += 1
        return value

    def read_bool(self) -> bool:
        value = self.read_u8()
        if value == 0:
            return False
        if value == 1:
            return True
        raise SoraReadError(f"invalid bool value {value}")

    def read_u32(self) -> int:
        if self._cursor + 4 > len(self._data):
            raise SoraReadError("Sora reader reached end of row")
        value = struct.unpack_from("<I", self._data, self._cursor)[0]
        self._cursor += 4
        return value

    def read_i32(self) -> int:
        if self._cursor + 4 > len(self._data):
            raise SoraReadError("Sora reader reached end of row")
        value = struct.unpack_from("<i", self._data, self._cursor)[0]
        self._cursor += 4
        return value

    def read_i64(self) -> int:
        if self._cursor + 8 > len(self._data):
            raise SoraReadError("Sora reader reached end of row")
        value = struct.unpack_from("<q", self._data, self._cursor)[0]
        self._cursor += 8
        return value

    def read_f32(self) -> float:
        if self._cursor + 4 > len(self._data):
            raise SoraReadError("Sora reader reached end of row")
        value = struct.unpack_from("<f", self._data, self._cursor)[0]
        self._cursor += 4
        return value

    def read_f64(self) -> float:
        if self._cursor + 8 > len(self._data):
            raise SoraReadError("Sora reader reached end of row")
        value = struct.unpack_from("<d", self._data, self._cursor)[0]
        self._cursor += 8
        return value

    def read_string(self) -> str:
        length = self.read_u32()
        if self._cursor + length > len(self._data):
            raise SoraReadError("Sora reader reached end of row")
        value = self._data[self._cursor : self._cursor + length].decode("utf-8")
        self._cursor += length
        return value

    def read_optional(self, read_fn: Callable[[], T]) -> T | None:
        presence = self.read_u8()
        if presence == 0:
            return None
        if presence == 1:
            return read_fn()
        raise SoraReadError(f"invalid option presence {presence}")

    def read_list(self, read_fn: Callable[[], T]) -> list[T]:
        length = self.read_u32()
        values = []
        for _ in range(length):
            values.append(read_fn())
        return values


def require_singleton_table(rows: list[T], name: str) -> T:
    if len(rows) != 1:
        raise SoraReadError(
            f"expected singleton table `{name}` to contain exactly 1 row, got {len(rows)}"
        )
    return rows[0]


def decode_map_table(rows: list[V], key_fn: Callable[[V], K]) -> dict[K, V]:
    values: dict[K, V] = {}
    for row in rows:
        key = key_fn(row)
        if key in values:
            raise SoraReadError(f"duplicate map key {key!r}")
        values[key] = row
    return values


def decode_unique_index(rows: list[V], key_fn: Callable[[V], K]) -> dict[K, V]:
    values: dict[K, V] = {}
    for row in rows:
        key = key_fn(row)
        if key in values:
            raise SoraReadError(f"duplicate unique index key {key!r}")
        values[key] = row
    return values


def decode_index(rows: list[V], key_fn: Callable[[V], K]) -> dict[K, list[V]]:
    values: dict[K, list[V]] = {}
    for row in rows:
        key = key_fn(row)
        if key not in values:
            values[key] = [row]
        else:
            values[key].append(row)
    return values


def decode_rows(payload: bytes, decode_fn: Callable[[SoraReader], T]) -> list[T]:
    if len(payload) < 12:
        raise SoraReadError("Sora table section is too short")

    row_count = struct.unpack_from("<I", payload, 0)[0]
    row_data_start = 4 + (row_count + 1) * 8
    if row_data_start > len(payload):
        raise SoraReadError("Sora row offset table exceeds payload length")

    rows = []
    for index in range(row_count):
        start = struct.unpack_from("<Q", payload, 4 + index * 8)[0]
        end = struct.unpack_from("<Q", payload, 4 + (index + 1) * 8)[0]
        if start > end:
            raise SoraReadError("Sora row offsets are not monotonic")
        absolute_start = row_data_start + start
        absolute_end = row_data_start + end
        if absolute_end > len(payload):
            raise SoraReadError("Sora row exceeds payload length")

        reader = SoraReader(payload[absolute_start:absolute_end])
        row = decode_fn(reader)
        if not reader.is_finished():
            raise SoraReadError("Sora row has trailing bytes")
        rows.append(row)
    return rows
