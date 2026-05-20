-module(element_type).

-export([decode/1]).
-export_type([t/0]).
-type t() ::
    'fire' |
    'ice' |
    'lightning' |
    'physical'.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Ordinal, Reader1} = sora_runtime:read_u32(Reader0),
    case Ordinal of
        0 -> {'fire', Reader1};
        1 -> {'ice', Reader1};
        2 -> {'lightning', Reader1};
        3 -> {'physical', Reader1};
        _ -> error({invalid_enum_ordinal, element_type, Ordinal})
    end.
