#include "stat_modifier.h"

sora_result sora_showcase_stat_modifier_decode(sora_reader* reader, sora_showcase_stat_modifier* out) {
    *out = (sora_showcase_stat_modifier){0};
    {
        sora_result result = sora_showcase_stat_type_decode(reader, &out->stat);
        if (result.code != SORA_OK) {
            sora_showcase_stat_modifier_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_f32(reader, &out->value);
        if (result.code != SORA_OK) {
            sora_showcase_stat_modifier_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_bool(reader, &out->is_percent);
        if (result.code != SORA_OK) {
            sora_showcase_stat_modifier_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_stat_modifier_free(sora_showcase_stat_modifier* value) {
    if (value == NULL) {
        return;
    }
    *value = (sora_showcase_stat_modifier){0};
}
