

export function decodeLocalization(reader) {
    return {
        key: reader.readString(),
        zhCn: reader.readString(),
        enUs: reader.readString(),
        note: reader.readOptional(() => reader.readString()),
    };
}
