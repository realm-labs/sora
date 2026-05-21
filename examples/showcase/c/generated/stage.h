#ifndef SORA_SHOWCASE_STAGE_H
#define SORA_SHOWCASE_STAGE_H

#include "sora_types.h"
#include "reward.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_stage {
    int32_t id;
    sora_string name;
    sora_showcase_i32_array monster_ids;
    int32_t recommended_power;
    sora_showcase_reward_array first_clear_rewards;
} sora_showcase_stage;

sora_result sora_showcase_stage_decode(sora_reader* reader, sora_showcase_stage* out);
void sora_showcase_stage_free(sora_showcase_stage* value);

#ifdef __cplusplus
}
#endif

#endif
