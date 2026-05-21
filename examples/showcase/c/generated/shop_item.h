#ifndef SORA_SHOWCASE_SHOP_ITEM_H
#define SORA_SHOWCASE_SHOP_ITEM_H

#include "sora_types.h"
#include "resource_cost.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_shop_item {
    int32_t shop_id;
    int32_t seq;
    int32_t item_id;
    sora_showcase_resource_cost price;
    sora_showcase_optional_i32 daily_limit;
} sora_showcase_shop_item;

sora_result sora_showcase_shop_item_decode(sora_reader* reader, sora_showcase_shop_item* out);
void sora_showcase_shop_item_free(sora_showcase_shop_item* value);

#ifdef __cplusplus
}
#endif

#endif
