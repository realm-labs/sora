#include "drop_group.h"

sora_result sora_showcase_drop_group_decode(sora_reader* reader, sora_showcase_drop_group* out) {
    *out = (sora_showcase_drop_group){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->id);
        if (result.code != SORA_OK) {
            sora_showcase_drop_group_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_string(reader, &out->name);
        if (result.code != SORA_OK) {
            sora_showcase_drop_group_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_drop_group_free(sora_showcase_drop_group* value) {
    if (value == NULL) {
        return;
    }
    sora_string_free(&value->name);
    *value = (sora_showcase_drop_group){0};
}
