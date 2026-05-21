#ifndef SORA_SHOWCASE_MAIL_TYPE_H
#define SORA_SHOWCASE_MAIL_TYPE_H

#include "sora_runtime.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum sora_showcase_mail_type {
    SORA_SHOWCASE_MAIL_TYPE_SYSTEM = 0,
    SORA_SHOWCASE_MAIL_TYPE_EVENT = 1,
    SORA_SHOWCASE_MAIL_TYPE_COMPENSATION = 2,
} sora_showcase_mail_type;

sora_result sora_showcase_mail_type_decode(sora_reader* reader, sora_showcase_mail_type* out);

#ifdef __cplusplus
}
#endif

#endif
