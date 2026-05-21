#ifndef SORA_SHOWCASE_REWARD_ACTION_H
#define SORA_SHOWCASE_REWARD_ACTION_H

#include "sora_types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum sora_showcase_reward_action_tag {
    SORA_SHOWCASE_REWARD_ACTION_ADD_ITEM = 0,
    SORA_SHOWCASE_REWARD_ACTION_ADD_BUFF = 1,
    SORA_SHOWCASE_REWARD_ACTION_UNLOCK_STAGE = 2,
    SORA_SHOWCASE_REWARD_ACTION_SEND_MAIL = 3,
} sora_showcase_reward_action_tag;

typedef struct sora_showcase_reward_action {
    sora_showcase_reward_action_tag tag;
    union {
        struct {
            int32_t item_id;
            int32_t count;
        } add_item;
        struct {
            int32_t buff_id;
            float duration;
        } add_buff;
        struct {
            int32_t stage_id;
        } unlock_stage;
        struct {
            int32_t mail_id;
        } send_mail;
    } value;
} sora_showcase_reward_action;

sora_result sora_showcase_reward_action_decode(sora_reader* reader, sora_showcase_reward_action* out);
void sora_showcase_reward_action_free(sora_showcase_reward_action* value);

#ifdef __cplusplus
}
#endif

#endif
