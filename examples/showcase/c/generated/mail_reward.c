#include "mail_reward.h"

sora_result sora_showcase_mail_reward_decode(sora_reader* reader, sora_showcase_mail_reward* out) {
    *out = (sora_showcase_mail_reward){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->mail_id);
        if (result.code != SORA_OK) {
            sora_showcase_mail_reward_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->seq);
        if (result.code != SORA_OK) {
            sora_showcase_mail_reward_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->item_id);
        if (result.code != SORA_OK) {
            sora_showcase_mail_reward_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->count);
        if (result.code != SORA_OK) {
            sora_showcase_mail_reward_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_mail_reward_free(sora_showcase_mail_reward* value) {
    if (value == NULL) {
        return;
    }
    *value = (sora_showcase_mail_reward){0};
}
