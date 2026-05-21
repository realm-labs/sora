#ifndef SORA_SHOWCASE_REWARD_H
#define SORA_SHOWCASE_REWARD_H

#include "sora_types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_reward {
    int32_t item_id;
    int32_t count;
} sora_showcase_reward;

sora_result sora_showcase_reward_decode(sora_reader* reader, sora_showcase_reward* out);
void sora_showcase_reward_free(sora_showcase_reward* value);

#ifdef __cplusplus
}
#endif

#endif
