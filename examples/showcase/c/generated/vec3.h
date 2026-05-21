#ifndef SORA_SHOWCASE_VEC3_H
#define SORA_SHOWCASE_VEC3_H

#include "sora_types.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct sora_showcase_vec3 {
    float x;
    float y;
    float z;
} sora_showcase_vec3;

sora_result sora_showcase_vec3_decode(sora_reader* reader, sora_showcase_vec3* out);
void sora_showcase_vec3_free(sora_showcase_vec3* value);

#ifdef __cplusplus
}
#endif

#endif
