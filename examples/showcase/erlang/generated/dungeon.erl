-module(dungeon).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'name' := binary(),
    'stage_ids' := [integer()],
    'entry_cost' := resource_cost:t()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Name, Reader2} = (fun sora_runtime:read_string/1)(Reader1),
    {StageIds, Reader3} = (fun(Reader) -> sora_runtime:read_list(fun sora_runtime:read_i32/1, Reader) end)(Reader2),
    {EntryCost, Reader4} = (fun resource_cost:decode/1)(Reader3),
    {#{
        'id' => Id,
        'name' => Name,
        'stage_ids' => StageIds,
        'entry_cost' => EntryCost
    }, Reader4}.
