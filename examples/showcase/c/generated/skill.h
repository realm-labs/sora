#ifndef SORA_SHOWCASE_SKILL_H
#define SORA_SHOWCASE_SKILL_H

#include "sora_types.h"
#include "element_type.h"
#include "resource_cost.h"
#include "skill_effect.h"
#include "vec3.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_skill {
    int32_t id;
    sora_string name;
    sora_showcase_element_type element;
    /* Tuple cost, e.g. Gold,0,150 */
    sora_showcase_resource_cost cost;
    /* JSON object with element/power/radius */
    sora_showcase_skill_effect effect;
    int32_t required_level;
    /* Optional item requirement */
    sora_showcase_optional_i32 required_item;
    sora_showcase_vec3 cast_origin;
} sora_showcase_skill;

sora_result sora_showcase_skill_decode(sora_reader* reader, sora_showcase_skill* out);
void sora_showcase_skill_free(sora_showcase_skill* value);

#ifdef __cplusplus
}
#endif

#endif
