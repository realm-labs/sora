#ifndef SORA_SHOWCASE_RECIPE_H
#define SORA_SHOWCASE_RECIPE_H

#include "sora_types.h"
#include "resource_cost.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_recipe {
    int32_t id;
    int32_t result_item;
    sora_showcase_resource_cost_array materials;
} sora_showcase_recipe;

sora_result sora_showcase_recipe_decode(sora_reader* reader, sora_showcase_recipe* out);
void sora_showcase_recipe_free(sora_showcase_recipe* value);

#ifdef __cplusplus
}
#endif

#endif
