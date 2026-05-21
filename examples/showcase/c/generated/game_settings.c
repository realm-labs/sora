#include "game_settings.h"

sora_result sora_showcase_game_settings_decode(sora_reader* reader, sora_showcase_game_settings* out) {
    *out = (sora_showcase_game_settings){0};
    {
        sora_result result = sora_reader_read_string(reader, &out->version);
        if (result.code != SORA_OK) {
            sora_showcase_game_settings_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->daily_reset_hour);
        if (result.code != SORA_OK) {
            sora_showcase_game_settings_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_reader_read_i32(reader, &out->starting_gold);
        if (result.code != SORA_OK) {
            sora_showcase_game_settings_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_vec3_decode(reader, &out->spawn_pos);
        if (result.code != SORA_OK) {
            sora_showcase_game_settings_free(out);
            return result;
        }
    }
    {
        sora_result result = sora_showcase_i32_array_decode(reader, &out->starter_items);
        if (result.code != SORA_OK) {
            sora_showcase_game_settings_free(out);
            return result;
        }
    }
    return sora_ok();
}

void sora_showcase_game_settings_free(sora_showcase_game_settings* value) {
    if (value == NULL) {
        return;
    }
    sora_string_free(&value->version);
    sora_showcase_vec3_free(&value->spawn_pos);
    sora_showcase_i32_array_free(&value->starter_items);
    *value = (sora_showcase_game_settings){0};
}
