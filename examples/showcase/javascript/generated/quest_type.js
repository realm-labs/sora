export const QuestType = {
    Main: "Main",
    Side: "Side",
    Daily: "Daily",
};
const values = [
    QuestType.Main,
    QuestType.Side,
    QuestType.Daily,
];

export function decodeQuestType(reader) {
    const ordinal = reader.readU32();
    const value = values[ordinal];
    if (value === undefined) {
        throw new Error(`invalid enum ordinal ${ordinal} for QuestType`);
    }
    return value;
}
