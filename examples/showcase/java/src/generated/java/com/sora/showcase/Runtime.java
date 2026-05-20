package com.sora.showcase;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.function.Supplier;

final class SoraReadException extends RuntimeException {
    SoraReadException(String message) {
        super(message);
    }
}

final class SoraSection {
    final int kind;
    final int compression;
    final String name;
    final int offset;
    final int length;
    final int uncompressedLength;

    SoraSection(int kind, int compression, String name, int offset, int length, int uncompressedLength) {
        this.kind = kind;
        this.compression = compression;
        this.name = name;
        this.offset = offset;
        this.length = length;
        this.uncompressedLength = uncompressedLength;
    }
}

final class SoraBundle {
    private static final int BUNDLE_VERSION = 1;
    private static final int HEADER_LENGTH = 24;
    private static final int SECTION_KIND_MANIFEST = 0;
    private static final int SECTION_KIND_SCHEMA = 1;
    private static final int SECTION_KIND_TABLE = 2;
    private static final int COMPRESSION_NONE = 0;

    private final byte[] bytes;
    private final List<SoraSection> sections;

    private SoraBundle(byte[] bytes, List<SoraSection> sections) {
        this.bytes = bytes;
        this.sections = sections;
    }

    <T> List<T> decodeTable(String name, SoraRowDecoder<T> decode) {
        var section = sections.stream()
            .filter(value -> value.kind == SECTION_KIND_TABLE && value.name.equals(name))
            .findFirst()
            .orElseThrow(() -> new SoraReadException("missing Sora table section `" + name + "`"));
        if (section.compression != COMPRESSION_NONE) {
            throw new SoraReadException("unsupported compression " + section.compression + " for table `" + name + "`");
        }
        if (section.uncompressedLength != section.length) {
            throw new SoraReadException("table `" + name + "` has invalid uncompressed length");
        }
        return decodeRows(Arrays.copyOfRange(bytes, section.offset, section.offset + section.length), decode);
    }

    static SoraBundle parse(byte[] bytes) {
        if (bytes.length < HEADER_LENGTH) {
            throw new SoraReadException("Sora bundle is shorter than header");
        }
        if (!new String(bytes, 0, 4, StandardCharsets.US_ASCII).equals("SORA")) {
            throw new SoraReadException("invalid Sora bundle magic");
        }
        var version = readI32At(bytes, 4);
        if (version != BUNDLE_VERSION) {
            throw new SoraReadException("unsupported Sora bundle version " + version);
        }

        var headerLength = readI32At(bytes, 8);
        var directoryLength = readI32At(bytes, 12);
        var sectionCount = readI32At(bytes, 16);
        var flags = readI32At(bytes, 20);
        if (flags != 0) {
            throw new SoraReadException("unsupported Sora bundle flags " + flags);
        }
        if (headerLength != HEADER_LENGTH || headerLength > bytes.length) {
            throw new SoraReadException("invalid Sora bundle header length");
        }
        var directoryEnd = checkedAdd(headerLength, directoryLength, "Sora section directory length overflow");
        if (directoryEnd > bytes.length) {
            throw new SoraReadException("Sora section directory exceeds bundle length");
        }

        var cursor = headerLength;
        var sections = new ArrayList<SoraSection>(sectionCount);
        var manifestCount = 0;
        var schemaCount = 0;
        var tableNames = new HashSet<String>();
        for (var i = 0; i < sectionCount; i++) {
            if (cursor + 40 > directoryEnd) {
                throw new SoraReadException("truncated Sora section directory entry");
            }
            var kind = readI32At(bytes, cursor);
            var compression = readI32At(bytes, cursor + 4);
            var nameLength = readI32At(bytes, cursor + 8);
            var entryFlags = readI32At(bytes, cursor + 12);
            var offset = checkedLongToInt(readI64At(bytes, cursor + 16), "Sora section offset exceeds Integer.MAX_VALUE");
            var length = checkedLongToInt(readI64At(bytes, cursor + 24), "Sora section length exceeds Integer.MAX_VALUE");
            var uncompressedLength = checkedLongToInt(readI64At(bytes, cursor + 32), "Sora section uncompressed length exceeds Integer.MAX_VALUE");
            if (entryFlags != 0) {
                throw new SoraReadException("unsupported Sora section flags " + entryFlags);
            }
            var nameStart = cursor + 40;
            var nameEnd = checkedAdd(nameStart, nameLength, "Sora section name length overflow");
            if (nameEnd > directoryEnd) {
                throw new SoraReadException("Sora section name exceeds directory");
            }
            var payloadEnd = checkedAdd(offset, length, "Sora section payload length overflow");
            if (payloadEnd > bytes.length) {
                throw new SoraReadException("Sora section payload exceeds bundle length");
            }
            var name = new String(bytes, nameStart, nameLength, StandardCharsets.UTF_8);
            if (kind == SECTION_KIND_MANIFEST) {
                manifestCount++;
                if (!name.equals("$manifest")) {
                    throw new SoraReadException("Sora manifest section must be named `$manifest`");
                }
            } else if (kind == SECTION_KIND_SCHEMA) {
                schemaCount++;
                if (!name.equals("$schema")) {
                    throw new SoraReadException("Sora schema section must be named `$schema`");
                }
            } else if (kind == SECTION_KIND_TABLE) {
                if (!tableNames.add(name)) {
                    throw new SoraReadException("duplicate Sora table section `" + name + "`");
                }
            } else {
                throw new SoraReadException("unknown Sora section kind " + kind);
            }
            if (compression == COMPRESSION_NONE && uncompressedLength != length) {
                throw new SoraReadException("uncompressed Sora section length mismatch");
            }
            sections.add(new SoraSection(kind, compression, name, offset, length, uncompressedLength));
            cursor = nameEnd;
        }
        if (cursor != directoryEnd) {
            throw new SoraReadException("Sora section directory has trailing bytes");
        }
        if (manifestCount != 1) {
            throw new SoraReadException("expected exactly 1 Sora manifest section, got " + manifestCount);
        }
        if (schemaCount != 1) {
            throw new SoraReadException("expected exactly 1 Sora schema section, got " + schemaCount);
        }
        return new SoraBundle(bytes, sections);
    }

    private static <T> List<T> decodeRows(byte[] payload, SoraRowDecoder<T> decode) {
        if (payload.length < 12) {
            throw new SoraReadException("Sora table section is too short");
        }
        var rowCount = readI32At(payload, 0);
        var offsetsLength = checkedAdd(rowCount, 1, "Sora row offset count overflow");
        var rowDataStart = checkedAdd(4, checkedMultiply(offsetsLength, 8, "Sora row offsets overflow"), "Sora row data offset overflow");
        if (rowDataStart > payload.length) {
            throw new SoraReadException("Sora row offset table exceeds payload length");
        }

        var rows = new ArrayList<T>(rowCount);
        for (var index = 0; index < rowCount; index++) {
            var start = checkedLongToInt(readI64At(payload, 4 + index * 8), "Sora row start exceeds Integer.MAX_VALUE");
            var end = checkedLongToInt(readI64At(payload, 4 + (index + 1) * 8), "Sora row end exceeds Integer.MAX_VALUE");
            if (start > end) {
                throw new SoraReadException("Sora row offsets are not monotonic");
            }
            var absoluteStart = checkedAdd(rowDataStart, start, "Sora row start overflow");
            var absoluteEnd = checkedAdd(rowDataStart, end, "Sora row end overflow");
            if (absoluteEnd > payload.length) {
                throw new SoraReadException("Sora row exceeds payload length");
            }
            var reader = new SoraReader(Arrays.copyOfRange(payload, absoluteStart, absoluteEnd));
            var row = decode.decode(reader);
            if (!reader.isFinished()) {
                throw new SoraReadException("Sora row has trailing bytes");
            }
            rows.add(row);
        }
        return rows;
    }

    static int readI32At(byte[] bytes, int offset) {
        if (offset + 4 > bytes.length) {
            throw new SoraReadException("unexpected end while reading i32");
        }
        return (bytes[offset] & 0xff) |
            ((bytes[offset + 1] & 0xff) << 8) |
            ((bytes[offset + 2] & 0xff) << 16) |
            ((bytes[offset + 3] & 0xff) << 24);
    }

    static long readI64At(byte[] bytes, int offset) {
        if (offset + 8 > bytes.length) {
            throw new SoraReadException("unexpected end while reading i64");
        }
        var value = 0L;
        for (var i = 0; i < 8; i++) {
            value |= ((long)bytes[offset + i] & 0xffL) << (i * 8);
        }
        return value;
    }

    static int checkedAdd(int left, int right, String message) {
        var value = (long)left + right;
        if (value > Integer.MAX_VALUE || value < Integer.MIN_VALUE) {
            throw new SoraReadException(message);
        }
        return (int)value;
    }

    private static int checkedMultiply(int left, int right, String message) {
        var value = (long)left * right;
        if (value > Integer.MAX_VALUE || value < Integer.MIN_VALUE) {
            throw new SoraReadException(message);
        }
        return (int)value;
    }

    private static int checkedLongToInt(long value, String message) {
        if (value > Integer.MAX_VALUE || value < Integer.MIN_VALUE) {
            throw new SoraReadException(message);
        }
        return (int)value;
    }
}

interface SoraRowDecoder<T> {
    T decode(SoraReader reader);
}

final class SoraReader {
    private final byte[] bytes;
    private int cursor;

    SoraReader(byte[] bytes) {
        this.bytes = bytes;
    }

    boolean isFinished() {
        return cursor == bytes.length;
    }

    int readU8() {
        return take(1)[0] & 0xff;
    }

    Boolean readBool() {
        switch (readU8()) {
            case 0:
                return false;
            case 1:
                return true;
            default:
                throw new SoraReadException("invalid bool value");
        }
    }

    int readU32() {
        return readI32();
    }

    Integer readI32() {
        return SoraBundle.readI32At(take(4), 0);
    }

    Long readI64() {
        return SoraBundle.readI64At(take(8), 0);
    }

    Float readF32() {
        return Float.intBitsToFloat(readI32());
    }

    Double readF64() {
        return Double.longBitsToDouble(readI64());
    }

    String readString() {
        var length = readU32();
        return new String(take(length), StandardCharsets.UTF_8);
    }

    <T> T readOptional(Supplier<T> read) {
        switch (readU8()) {
            case 0:
                return null;
            case 1:
                return read.get();
            default:
                throw new SoraReadException("invalid option presence");
        }
    }

    <T> List<T> readList(Supplier<T> read) {
        var length = readU32();
        var values = new ArrayList<T>(length);
        for (var i = 0; i < length; i++) {
            values.add(read.get());
        }
        return values;
    }

    private byte[] take(int length) {
        var end = SoraBundle.checkedAdd(cursor, length, "Sora reader cursor overflow");
        if (end > bytes.length) {
            throw new SoraReadException("Sora reader reached end of row");
        }
        var chunk = Arrays.copyOfRange(bytes, cursor, end);
        cursor = end;
        return chunk;
    }
}
