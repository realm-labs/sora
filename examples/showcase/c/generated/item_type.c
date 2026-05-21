#include "item_type.h"

sora_result sora_showcase_item_type_decode(sora_reader* reader, sora_showcase_item_type* out) {
    uint32_t ordinal = 0;
    SORA_TRY(sora_reader_read_u32(reader, &ordinal));
    switch (ordinal) {
    case 0:
        *out = SORA_SHOWCASE_ITEM_TYPE_WEAPON;
        return sora_ok();
    case 1:
        *out = SORA_SHOWCASE_ITEM_TYPE_ARMOR;
        return sora_ok();
    case 2:
        *out = SORA_SHOWCASE_ITEM_TYPE_CURRENCY;
        return sora_ok();
    case 3:
        *out = SORA_SHOWCASE_ITEM_TYPE_MATERIAL;
        return sora_ok();
    case 4:
        *out = SORA_SHOWCASE_ITEM_TYPE_CONSUMABLE;
        return sora_ok();
    default:
        return sora_error(SORA_ERROR_DECODE, "invalid enum ordinal for ItemType");
    }
}
