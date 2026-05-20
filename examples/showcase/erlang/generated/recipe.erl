-module(recipe).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'result_item' := integer(),
    'materials' := [resource_cost:t()]
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {ResultItem, Reader2} = (fun sora_runtime:read_i32/1)(Reader1),
    {Materials, Reader3} = (fun(Reader) -> sora_runtime:read_list(fun resource_cost:decode/1, Reader) end)(Reader2),
    {#{
        'id' => Id,
        'result_item' => ResultItem,
        'materials' => Materials
    }, Reader3}.
