#include "vip_level.h"

sora_result sora_showcase_vip_level_decode(sora_reader* reader, sora_showcase_vip_level* out) {
    *out = (sora_showcase_vip_level){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->level);
        if (result.code != SORA_OK) {
            sora_showcase_vip_level_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_resource_cost_decode(reader, &out->cost);
        if (result.code != SORA_OK) {
            sora_showcase_vip_level_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_string_array_decode(reader, &out->perks);
        if (result.code != SORA_OK) {
            sora_showcase_vip_level_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_vip_level_free(sora_showcase_vip_level* value) {
    if (value == NULL) {
        return;
    }
    sora_showcase_resource_cost_free(&value->cost);
    sora_showcase_string_array_free(&value->perks);
    *value = (sora_showcase_vip_level){0};
}
