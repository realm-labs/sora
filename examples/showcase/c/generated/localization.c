#include "localization.h"

sora_result sora_showcase_localization_decode(sora_reader* reader, sora_showcase_localization* out) {
    *out = (sora_showcase_localization){0};
    {
        sora_result result = sora_reader_read_string(reader, &out->key);
        if (result.code != SORA_OK) {
            sora_showcase_localization_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_string(reader, &out->zh_cn);
        if (result.code != SORA_OK) {
            sora_showcase_localization_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_string(reader, &out->en_us);
        if (result.code != SORA_OK) {
            sora_showcase_localization_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_optional_string_decode(reader, &out->note);
        if (result.code != SORA_OK) {
            sora_showcase_localization_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_localization_free(sora_showcase_localization* value) {
    if (value == NULL) {
        return;
    }
    sora_string_free(&value->key);
    sora_string_free(&value->zh_cn);
    sora_string_free(&value->en_us);
    sora_showcase_optional_string_free(&value->note);
    *value = (sora_showcase_localization){0};
}
