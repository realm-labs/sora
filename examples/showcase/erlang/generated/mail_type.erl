-module(mail_type).

-export([decode/1]).
-export_type([t/0]).
-type t() ::
    'system' |
    'event' |
    'compensation'.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Ordinal, Reader1} = sora_runtime:read_u32(Reader0),
    case Ordinal of
        0 -> {'system', Reader1};
        1 -> {'event', Reader1};
        2 -> {'compensation', Reader1};
        _ -> error({invalid_enum_ordinal, mail_type, Ordinal})
    end.
