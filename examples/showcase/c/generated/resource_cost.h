#ifndef SORA_SHOWCASE_RESOURCE_COST_H
#define SORA_SHOWCASE_RESOURCE_COST_H

#include "sora_types.h"
#include "resource_kind.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_resource_cost {
    sora_showcase_resource_kind kind;
    int32_t id;
    int32_t count;
} sora_showcase_resource_cost;

sora_result sora_showcase_resource_cost_decode(sora_reader* reader, sora_showcase_resource_cost* out);
void sora_showcase_resource_cost_free(sora_showcase_resource_cost* value);

#ifdef __cplusplus
}
#endif

#endif
