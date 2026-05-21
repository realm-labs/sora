#ifndef SORA_SHOWCASE_STAT_MODIFIER_H
#define SORA_SHOWCASE_STAT_MODIFIER_H

#include "sora_types.h"
#include "stat_type.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_stat_modifier {
    sora_showcase_stat_type stat;
    float value;
    bool is_percent;
} sora_showcase_stat_modifier;

sora_result sora_showcase_stat_modifier_decode(sora_reader* reader, sora_showcase_stat_modifier* out);
void sora_showcase_stat_modifier_free(sora_showcase_stat_modifier* value);

#ifdef __cplusplus
}
#endif

#endif
