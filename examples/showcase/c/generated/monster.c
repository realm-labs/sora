#include "monster.h"

sora_result sora_showcase_monster_decode(sora_reader* reader, sora_showcase_monster* out) {
    *out = (sora_showcase_monster){0};
    {
        sora_result result = sora_reader_read_i32(reader, &out->id);
        if (result.code != SORA_OK) {
            sora_showcase_monster_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_string(reader, &out->name);
        if (result.code != SORA_OK) {
            sora_showcase_monster_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->level);
        if (result.code != SORA_OK) {
            sora_showcase_monster_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_element_type_decode(reader, &out->element);
        if (result.code != SORA_OK) {
            sora_showcase_monster_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->drop_group);
        if (result.code != SORA_OK) {
            sora_showcase_monster_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_vec3_decode(reader, &out->spawn_pos);
        if (result.code != SORA_OK) {
            sora_showcase_monster_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_monster_free(sora_showcase_monster* value) {
    if (value == NULL) {
        return;
    }
    sora_string_free(&value->name);
    sora_showcase_vec3_free(&value->spawn_pos);
    *value = (sora_showcase_monster){0};
}
