#include "equipment_set.h"

sora_result sora_showcase_equipment_set_decode(sora_reader* reader, sora_showcase_equipment_set* out) {
    *out = (sora_showcase_equipment_set){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->id);
        if (result.code != SORA_OK) {
            sora_showcase_equipment_set_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_string(reader, &out->name);
        if (result.code != SORA_OK) {
            sora_showcase_equipment_set_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_i32_array_decode(reader, &out->item_ids);
        if (result.code != SORA_OK) {
            sora_showcase_equipment_set_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_skill_effect_decode(reader, &out->bonus_effect);
        if (result.code != SORA_OK) {
            sora_showcase_equipment_set_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_equipment_set_free(sora_showcase_equipment_set* value) {
    if (value == NULL) {
        return;
    }
    sora_string_free(&value->name);
    sora_showcase_i32_array_free(&value->item_ids);
    sora_showcase_skill_effect_free(&value->bonus_effect);
    *value = (sora_showcase_equipment_set){0};
}
