#include "gacha_item.h"

sora_result sora_showcase_gacha_item_decode(sora_reader* reader, sora_showcase_gacha_item* out) {
    *out = (sora_showcase_gacha_item){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->pool_id);
        if (result.code != SORA_OK) {
            sora_showcase_gacha_item_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->item_id);
        if (result.code != SORA_OK) {
            sora_showcase_gacha_item_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_rarity_decode(reader, &out->rarity);
        if (result.code != SORA_OK) {
            sora_showcase_gacha_item_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_f32(reader, &out->weight);
        if (result.code != SORA_OK) {
            sora_showcase_gacha_item_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_gacha_item_free(sora_showcase_gacha_item* value) {
    if (value == NULL) {
        return;
    }
    *value = (sora_showcase_gacha_item){0};
}
