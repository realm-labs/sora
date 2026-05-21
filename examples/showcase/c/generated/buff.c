#include "buff.h"

sora_result sora_showcase_buff_decode(sora_reader* reader, sora_showcase_buff* out) {
    *out = (sora_showcase_buff){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->id);
        if (result.code != SORA_OK) {
            sora_showcase_buff_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_string(reader, &out->name);
        if (result.code != SORA_OK) {
            sora_showcase_buff_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_f32(reader, &out->duration);
        if (result.code != SORA_OK) {
            sora_showcase_buff_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_stat_modifier_array_decode(reader, &out->modifiers);
        if (result.code != SORA_OK) {
            sora_showcase_buff_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_buff_free(sora_showcase_buff* value) {
    if (value == NULL) {
        return;
    }
    sora_string_free(&value->name);
    sora_showcase_stat_modifier_array_free(&value->modifiers);
    *value = (sora_showcase_buff){0};
}
