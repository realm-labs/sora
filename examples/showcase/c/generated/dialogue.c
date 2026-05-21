#include "dialogue.h"

sora_result sora_showcase_dialogue_decode(sora_reader* reader, sora_showcase_dialogue* out) {
    *out = (sora_showcase_dialogue){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->id);
        if (result.code != SORA_OK) {
            sora_showcase_dialogue_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_string(reader, &out->speaker_key);
        if (result.code != SORA_OK) {
            sora_showcase_dialogue_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_string_array_decode(reader, &out->lines);
        if (result.code != SORA_OK) {
            sora_showcase_dialogue_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_dialogue_free(sora_showcase_dialogue* value) {
    if (value == NULL) {
        return;
    }
    sora_string_free(&value->speaker_key);
    sora_showcase_string_array_free(&value->lines);
    *value = (sora_showcase_dialogue){0};
}
