#ifndef SORA_SHOWCASE_MAIL_TEMPLATE_H
#define SORA_SHOWCASE_MAIL_TEMPLATE_H

#include "sora_types.h"
#include "mail_type.h"
#include "reward.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_mail_template {
    int32_t id;
    sora_showcase_mail_type mail_type;
    sora_string title_key;
    sora_string body_key;
    sora_showcase_reward_array rewards;
} sora_showcase_mail_template;

sora_result sora_showcase_mail_template_decode(sora_reader* reader, sora_showcase_mail_template* out);
void sora_showcase_mail_template_free(sora_showcase_mail_template* value);

#ifdef __cplusplus
}
#endif

#endif
