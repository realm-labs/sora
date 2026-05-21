#ifndef SORA_SHOWCASE_VIP_LEVEL_H
#define SORA_SHOWCASE_VIP_LEVEL_H

#include "sora_types.h"
#include "resource_cost.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_vip_level {
    int32_t level;
    sora_showcase_resource_cost cost;
    sora_showcase_string_array perks;
} sora_showcase_vip_level;

sora_result sora_showcase_vip_level_decode(sora_reader* reader, sora_showcase_vip_level* out);
void sora_showcase_vip_level_free(sora_showcase_vip_level* value);

#ifdef __cplusplus
}
#endif

#endif
