#include "quest_type.h"

sora_result sora_showcase_quest_type_decode(sora_reader* reader, sora_showcase_quest_type* out) {
    uint32_t ordinal = 0;
    SORA_TRY(sora_reader_read_u32(reader, &ordinal));
    switch (ordinal) {
    case 0:
        *out = SORA_SHOWCASE_QUEST_TYPE_MAIN;
        return sora_ok();
    case 1:
        *out = SORA_SHOWCASE_QUEST_TYPE_SIDE;
        return sora_ok();
    case 2:
        *out = SORA_SHOWCASE_QUEST_TYPE_DAILY;
        return sora_ok();
    default:
        return sora_error(SORA_ERROR_DECODE, "invalid enum ordinal for QuestType");
    }
}
