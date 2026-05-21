#include "recipe.h"

sora_result sora_showcase_recipe_decode(sora_reader* reader, sora_showcase_recipe* out) {
    *out = (sora_showcase_recipe){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->id);
        if (result.code != SORA_OK) {
            sora_showcase_recipe_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->result_item);
        if (result.code != SORA_OK) {
            sora_showcase_recipe_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_resource_cost_array_decode(reader, &out->materials);
        if (result.code != SORA_OK) {
            sora_showcase_recipe_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_recipe_free(sora_showcase_recipe* value) {
    if (value == NULL) {
        return;
    }
    sora_showcase_resource_cost_array_free(&value->materials);
    *value = (sora_showcase_recipe){0};
}
