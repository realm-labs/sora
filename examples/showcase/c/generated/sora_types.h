#ifndef SORA_SHOWCASE_SORA_TYPES_H
#define SORA_SHOWCASE_SORA_TYPES_H

#include "sora_runtime.h"
#include "item_type.h"
#include "resource_kind.h"
#include "element_type.h"
#include "quest_type.h"
#include "rarity.h"
#include "stat_type.h"
#include "mail_type.h"

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif
typedef struct sora_showcase_resource_cost sora_showcase_resource_cost;
typedef struct sora_showcase_vec3 sora_showcase_vec3;
typedef struct sora_showcase_skill_effect sora_showcase_skill_effect;
typedef struct sora_showcase_reward sora_showcase_reward;
typedef struct sora_showcase_stat_modifier sora_showcase_stat_modifier;
typedef struct sora_showcase_item sora_showcase_item;
typedef struct sora_showcase_skill sora_showcase_skill;
typedef struct sora_showcase_quest sora_showcase_quest;
typedef struct sora_showcase_quest_reward sora_showcase_quest_reward;
typedef struct sora_showcase_game_settings sora_showcase_game_settings;
typedef struct sora_showcase_localization sora_showcase_localization;
typedef struct sora_showcase_level_exp sora_showcase_level_exp;
typedef struct sora_showcase_character sora_showcase_character;
typedef struct sora_showcase_character_skill sora_showcase_character_skill;
typedef struct sora_showcase_buff sora_showcase_buff;
typedef struct sora_showcase_drop_group sora_showcase_drop_group;
typedef struct sora_showcase_drop_entry sora_showcase_drop_entry;
typedef struct sora_showcase_monster sora_showcase_monster;
typedef struct sora_showcase_stage sora_showcase_stage;
typedef struct sora_showcase_stage_reward sora_showcase_stage_reward;
typedef struct sora_showcase_dungeon sora_showcase_dungeon;
typedef struct sora_showcase_shop sora_showcase_shop;
typedef struct sora_showcase_shop_item sora_showcase_shop_item;
typedef struct sora_showcase_recipe sora_showcase_recipe;
typedef struct sora_showcase_gacha_pool sora_showcase_gacha_pool;
typedef struct sora_showcase_gacha_item sora_showcase_gacha_item;
typedef struct sora_showcase_equipment_set sora_showcase_equipment_set;
typedef struct sora_showcase_achievement sora_showcase_achievement;
typedef struct sora_showcase_vip_level sora_showcase_vip_level;
typedef struct sora_showcase_mail_template sora_showcase_mail_template;
typedef struct sora_showcase_mail_reward sora_showcase_mail_reward;
typedef struct sora_showcase_dialogue sora_showcase_dialogue;
typedef struct sora_showcase_event_rule sora_showcase_event_rule;
typedef struct sora_showcase_event_condition sora_showcase_event_condition;
typedef struct sora_showcase_reward_action sora_showcase_reward_action;

typedef struct sora_showcase_i32_array {
    int32_t* data;
    size_t len;
} sora_showcase_i32_array;

sora_result sora_showcase_i32_array_decode(sora_reader* reader, sora_showcase_i32_array* out);
void sora_showcase_i32_array_free(sora_showcase_i32_array* value);

typedef struct sora_showcase_optional_i32 {
    bool has_value;
    int32_t* value;
} sora_showcase_optional_i32;

sora_result sora_showcase_optional_i32_decode(sora_reader* reader, sora_showcase_optional_i32* out);
void sora_showcase_optional_i32_free(sora_showcase_optional_i32* value);

typedef struct sora_showcase_optional_string {
    bool has_value;
    sora_string* value;
} sora_showcase_optional_string;

sora_result sora_showcase_optional_string_decode(sora_reader* reader, sora_showcase_optional_string* out);
void sora_showcase_optional_string_free(sora_showcase_optional_string* value);

typedef struct sora_showcase_resource_cost_array {
    sora_showcase_resource_cost* data;
    size_t len;
} sora_showcase_resource_cost_array;

sora_result sora_showcase_resource_cost_array_decode(sora_reader* reader, sora_showcase_resource_cost_array* out);
void sora_showcase_resource_cost_array_free(sora_showcase_resource_cost_array* value);

typedef struct sora_showcase_reward_action_array {
    sora_showcase_reward_action* data;
    size_t len;
} sora_showcase_reward_action_array;

sora_result sora_showcase_reward_action_array_decode(sora_reader* reader, sora_showcase_reward_action_array* out);
void sora_showcase_reward_action_array_free(sora_showcase_reward_action_array* value);

typedef struct sora_showcase_reward_array {
    sora_showcase_reward* data;
    size_t len;
} sora_showcase_reward_array;

sora_result sora_showcase_reward_array_decode(sora_reader* reader, sora_showcase_reward_array* out);
void sora_showcase_reward_array_free(sora_showcase_reward_array* value);

typedef struct sora_showcase_stat_modifier_array {
    sora_showcase_stat_modifier* data;
    size_t len;
} sora_showcase_stat_modifier_array;

sora_result sora_showcase_stat_modifier_array_decode(sora_reader* reader, sora_showcase_stat_modifier_array* out);
void sora_showcase_stat_modifier_array_free(sora_showcase_stat_modifier_array* value);

typedef struct sora_showcase_string_array {
    sora_string* data;
    size_t len;
} sora_showcase_string_array;

sora_result sora_showcase_string_array_decode(sora_reader* reader, sora_showcase_string_array* out);
void sora_showcase_string_array_free(sora_showcase_string_array* value);

#ifdef __cplusplus
}
#endif

#endif
