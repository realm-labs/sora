#ifndef SORA_SHOWCASE_STAT_TYPE_H
#define SORA_SHOWCASE_STAT_TYPE_H

#include "sora_runtime.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum sora_showcase_stat_type {
    SORA_SHOWCASE_STAT_TYPE_HP = 0,
    SORA_SHOWCASE_STAT_TYPE_ATTACK = 1,
    SORA_SHOWCASE_STAT_TYPE_DEFENSE = 2,
    SORA_SHOWCASE_STAT_TYPE_SPEED = 3,
    SORA_SHOWCASE_STAT_TYPE_CRIT_RATE = 4,
} sora_showcase_stat_type;

sora_result sora_showcase_stat_type_decode(sora_reader* reader, sora_showcase_stat_type* out);

#ifdef __cplusplus
}
#endif

#endif
