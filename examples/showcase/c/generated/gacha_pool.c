#include "gacha_pool.h"

sora_result sora_showcase_gacha_pool_decode(sora_reader* reader, sora_showcase_gacha_pool* out) {
    *out = (sora_showcase_gacha_pool){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->id);
        if (result.code != SORA_OK) {
            sora_showcase_gacha_pool_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_string(reader, &out->name);
        if (result.code != SORA_OK) {
            sora_showcase_gacha_pool_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_resource_cost_decode(reader, &out->cost);
        if (result.code != SORA_OK) {
            sora_showcase_gacha_pool_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_gacha_pool_free(sora_showcase_gacha_pool* value) {
    if (value == NULL) {
        return;
    }
    sora_string_free(&value->name);
    sora_showcase_resource_cost_free(&value->cost);
    *value = (sora_showcase_gacha_pool){0};
}
