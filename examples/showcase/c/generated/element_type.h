#ifndef SORA_SHOWCASE_ELEMENT_TYPE_H
#define SORA_SHOWCASE_ELEMENT_TYPE_H

#include "sora_runtime.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum sora_showcase_element_type {
    SORA_SHOWCASE_ELEMENT_TYPE_FIRE = 0,
    SORA_SHOWCASE_ELEMENT_TYPE_ICE = 1,
    SORA_SHOWCASE_ELEMENT_TYPE_LIGHTNING = 2,
    SORA_SHOWCASE_ELEMENT_TYPE_PHYSICAL = 3,
} sora_showcase_element_type;

sora_result sora_showcase_element_type_decode(sora_reader* reader, sora_showcase_element_type* out);

#ifdef __cplusplus
}
#endif

#endif
