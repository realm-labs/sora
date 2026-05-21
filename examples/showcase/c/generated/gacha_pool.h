#ifndef SORA_SHOWCASE_GACHA_POOL_H
#define SORA_SHOWCASE_GACHA_POOL_H

#include "sora_types.h"
#include "resource_cost.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_gacha_pool {
    int32_t id;
    sora_string name;
    sora_showcase_resource_cost cost;
} sora_showcase_gacha_pool;

sora_result sora_showcase_gacha_pool_decode(sora_reader* reader, sora_showcase_gacha_pool* out);
void sora_showcase_gacha_pool_free(sora_showcase_gacha_pool* value);

#ifdef __cplusplus
}
#endif

#endif
