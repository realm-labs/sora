#ifndef SORA_SHOWCASE_EVENT_CONDITION_H
#define SORA_SHOWCASE_EVENT_CONDITION_H

#include "sora_types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum sora_showcase_event_condition_tag {
    SORA_SHOWCASE_EVENT_CONDITION_LEVEL_AT_LEAST = 0,
    SORA_SHOWCASE_EVENT_CONDITION_QUEST_COMPLETED = 1,
    SORA_SHOWCASE_EVENT_CONDITION_HAS_ITEM = 2,
} sora_showcase_event_condition_tag;

typedef struct sora_showcase_event_condition {
    sora_showcase_event_condition_tag tag;
    union {
        struct {
            int32_t level;
        } level_at_least;
        struct {
            int32_t quest_id;
        } quest_completed;
        struct {
            int32_t item_id;
            int32_t count;
        } has_item;
    } value;
} sora_showcase_event_condition;

sora_result sora_showcase_event_condition_decode(sora_reader* reader, sora_showcase_event_condition* out);
void sora_showcase_event_condition_free(sora_showcase_event_condition* value);

#ifdef __cplusplus
}
#endif

#endif
