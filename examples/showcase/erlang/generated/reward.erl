-module(reward).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'item_id' := integer(),
    'count' := integer()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {ItemId, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Count, Reader2} = (fun sora_runtime:read_i32/1)(Reader1),
    {#{
        'item_id' => ItemId,
        'count' => Count
    }, Reader2}.
