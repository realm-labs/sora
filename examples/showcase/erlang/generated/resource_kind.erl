-module(resource_kind).

-export([decode/1]).
-export_type([t/0]).
-type t() ::
    'item' |
    'gold' |
    'diamond'.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Ordinal, Reader1} = sora_runtime:read_u32(Reader0),
    case Ordinal of
        0 -> {'item', Reader1};
        1 -> {'gold', Reader1};
        2 -> {'diamond', Reader1};
        _ -> error({invalid_enum_ordinal, resource_kind, Ordinal})
    end.
