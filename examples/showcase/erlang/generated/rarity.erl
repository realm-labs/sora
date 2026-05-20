-module(rarity).

-export([decode/1]).
-export_type([t/0]).
-type t() ::
    'common' |
    'uncommon' |
    'rare' |
    'epic' |
    'legendary'.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Ordinal, Reader1} = sora_runtime:read_u32(Reader0),
    case Ordinal of
        0 -> {'common', Reader1};
        1 -> {'uncommon', Reader1};
        2 -> {'rare', Reader1};
        3 -> {'epic', Reader1};
        4 -> {'legendary', Reader1};
        _ -> error({invalid_enum_ordinal, rarity, Ordinal})
    end.
