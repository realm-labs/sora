-module(stat_type).

-export([decode/1]).
-export_type([t/0]).
-type t() ::
    'hp' |
    'attack' |
    'defense' |
    'speed' |
    'crit_rate'.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Ordinal, Reader1} = sora_runtime:read_u32(Reader0),
    case Ordinal of
        0 -> {'hp', Reader1};
        1 -> {'attack', Reader1};
        2 -> {'defense', Reader1};
        3 -> {'speed', Reader1};
        4 -> {'crit_rate', Reader1};
        _ -> error({invalid_enum_ordinal, stat_type, Ordinal})
    end.
