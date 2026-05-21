#ifndef SORA_SHOWCASE_ACHIEVEMENT_H
#define SORA_SHOWCASE_ACHIEVEMENT_H

#include "sora_types.h"
#include "resource_cost.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_achievement {
    int32_t id;
    sora_string title_key;
    int64_t target_count;
    sora_showcase_resource_cost reward;
} sora_showcase_achievement;

sora_result sora_showcase_achievement_decode(sora_reader* reader, sora_showcase_achievement* out);
void sora_showcase_achievement_free(sora_showcase_achievement* value);

#ifdef __cplusplus
}
#endif

#endif
