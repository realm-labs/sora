#include "rarity.h"

sora_result sora_showcase_rarity_decode(sora_reader* reader, sora_showcase_rarity* out) {
    uint32_t ordinal = 0;
    SORA_TRY(sora_reader_read_u32(reader, &ordinal));
    switch (ordinal) {
    case 0:
        *out = SORA_SHOWCASE_RARITY_COMMON;
        return sora_ok();
    case 1:
        *out = SORA_SHOWCASE_RARITY_UNCOMMON;
        return sora_ok();
    case 2:
        *out = SORA_SHOWCASE_RARITY_RARE;
        return sora_ok();
    case 3:
        *out = SORA_SHOWCASE_RARITY_EPIC;
        return sora_ok();
    case 4:
        *out = SORA_SHOWCASE_RARITY_LEGENDARY;
        return sora_ok();
    default:
        return sora_error(SORA_ERROR_DECODE, "invalid enum ordinal for Rarity");
    }
}
