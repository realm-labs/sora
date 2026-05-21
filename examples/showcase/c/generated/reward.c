#include "reward.h"

sora_result sora_showcase_reward_decode(sora_reader* reader, sora_showcase_reward* out) {
    *out = (sora_showcase_reward){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->item_id);
        if (result.code != SORA_OK) {
            sora_showcase_reward_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->count);
        if (result.code != SORA_OK) {
            sora_showcase_reward_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_reward_free(sora_showcase_reward* value) {
    if (value == NULL) {
        return;
    }
    *value = (sora_showcase_reward){0};
}
