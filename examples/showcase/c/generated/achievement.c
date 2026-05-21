#include "achievement.h"

sora_result sora_showcase_achievement_decode(sora_reader* reader, sora_showcase_achievement* out) {
    *out = (sora_showcase_achievement){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->id);
        if (result.code != SORA_OK) {
            sora_showcase_achievement_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_string(reader, &out->title_key);
        if (result.code != SORA_OK) {
            sora_showcase_achievement_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i64(reader, &out->target_count);
        if (result.code != SORA_OK) {
            sora_showcase_achievement_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_resource_cost_decode(reader, &out->reward);
        if (result.code != SORA_OK) {
            sora_showcase_achievement_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_achievement_free(sora_showcase_achievement* value) {
    if (value == NULL) {
        return;
    }
    sora_string_free(&value->title_key);
    sora_showcase_resource_cost_free(&value->reward);
    *value = (sora_showcase_achievement){0};
}
