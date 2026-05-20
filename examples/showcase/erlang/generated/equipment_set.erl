-module(equipment_set).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'name' := binary(),
    'item_ids' := [integer()],
    'bonus_effect' := skill_effect:t()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1 } = (fun sora_runtime:read_i32/1)(Reader0),
    {Name, Reader2 } = (fun sora_runtime:read_string/1)(Reader1),
    {ItemIds, Reader3 } = (fun(Reader) -> sora_runtime:read_list(fun sora_runtime:read_i32/1, Reader) end)(Reader2),
    {BonusEffect, Reader4 } = (fun skill_effect:decode/1)(Reader3),
    { #{
        'id' => Id,
        'name' => Name,
        'item_ids' => ItemIds,
        'bonus_effect' => BonusEffect
    }, Reader4}.
