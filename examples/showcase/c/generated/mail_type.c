#include "mail_type.h"

sora_result sora_showcase_mail_type_decode(sora_reader* reader, sora_showcase_mail_type* out) {
    uint32_t ordinal = 0;
    SORA_TRY(sora_reader_read_u32(reader, &ordinal));
    switch (ordinal) {
    case 0:
        *out = SORA_SHOWCASE_MAIL_TYPE_SYSTEM;
        return sora_ok();
    case 1:
        *out = SORA_SHOWCASE_MAIL_TYPE_EVENT;
        return sora_ok();
    case 2:
        *out = SORA_SHOWCASE_MAIL_TYPE_COMPENSATION;
        return sora_ok();
    default:
        return sora_error(SORA_ERROR_DECODE, "invalid enum ordinal for MailType");
    }
}
