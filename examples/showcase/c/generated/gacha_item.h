#ifndef SORA_SHOWCASE_GACHA_ITEM_H
#define SORA_SHOWCASE_GACHA_ITEM_H

#include "sora_types.h"
#include "rarity.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_gacha_item {
    int32_t pool_id;
    int32_t item_id;
    sora_showcase_rarity rarity;
    float weight;
} sora_showcase_gacha_item;

sora_result sora_showcase_gacha_item_decode(sora_reader* reader, sora_showcase_gacha_item* out);
void sora_showcase_gacha_item_free(sora_showcase_gacha_item* value);

#ifdef __cplusplus
}
#endif

#endif
