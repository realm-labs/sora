#include "skill.h"

sora_result sora_showcase_skill_decode(sora_reader* reader, sora_showcase_skill* out) {
    *out = (sora_showcase_skill){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->id);
        if (result.code != SORA_OK) {
            sora_showcase_skill_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_string(reader, &out->name);
        if (result.code != SORA_OK) {
            sora_showcase_skill_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_element_type_decode(reader, &out->element);
        if (result.code != SORA_OK) {
            sora_showcase_skill_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_resource_cost_decode(reader, &out->cost);
        if (result.code != SORA_OK) {
            sora_showcase_skill_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_skill_effect_decode(reader, &out->effect);
        if (result.code != SORA_OK) {
            sora_showcase_skill_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->required_level);
        if (result.code != SORA_OK) {
            sora_showcase_skill_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_optional_i32_decode(reader, &out->required_item);
        if (result.code != SORA_OK) {
            sora_showcase_skill_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_vec3_decode(reader, &out->cast_origin);
        if (result.code != SORA_OK) {
            sora_showcase_skill_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_skill_free(sora_showcase_skill* value) {
    if (value == NULL) {
        return;
    }
    sora_string_free(&value->name);
    sora_showcase_resource_cost_free(&value->cost);
    sora_showcase_skill_effect_free(&value->effect);
    sora_showcase_optional_i32_free(&value->required_item);
    sora_showcase_vec3_free(&value->cast_origin);
    *value = (sora_showcase_skill){0};
}
