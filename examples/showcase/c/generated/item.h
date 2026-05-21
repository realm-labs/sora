#ifndef SORA_SHOWCASE_ITEM_H
#define SORA_SHOWCASE_ITEM_H

#include "sora_types.h"
#include "item_type.h"
#include "resource_cost.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_item {
    /* Item id */
    int32_t id;
    /* Display name */
    sora_string name;
    /* Item category */
    sora_showcase_item_type item_type;
    /* Stack limit; blank cells use the default */
    int32_t max_stack;
    /* Tuple: kind,id,count */
    sora_showcase_resource_cost price;
    /* JSON string array */
    sora_showcase_string_array tags;
} sora_showcase_item;

sora_result sora_showcase_item_decode(sora_reader* reader, sora_showcase_item* out);
void sora_showcase_item_free(sora_showcase_item* value);

#ifdef __cplusplus
}
#endif

#endif
