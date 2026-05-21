#ifndef SORA_SHOWCASE_SHOP_H
#define SORA_SHOWCASE_SHOP_H

#include "sora_types.h"
#include "resource_kind.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_shop {
    int32_t id;
    sora_string name;
    sora_showcase_resource_kind currency;
} sora_showcase_shop;

sora_result sora_showcase_shop_decode(sora_reader* reader, sora_showcase_shop* out);
void sora_showcase_shop_free(sora_showcase_shop* value);

#ifdef __cplusplus
}
#endif

#endif
