-module(stage).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'name' := binary(),
    'monster_ids' := [integer()],
    'recommended_power' := integer(),
    'first_clear_rewards' := [reward:t()]
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Name, Reader2} = (fun sora_runtime:read_string/1)(Reader1),
    {MonsterIds, Reader3} = (fun(Reader) -> sora_runtime:read_list(fun sora_runtime:read_i32/1, Reader) end)(Reader2),
    {RecommendedPower, Reader4} = (fun sora_runtime:read_i32/1)(Reader3),
    {FirstClearRewards, Reader5} = (fun(Reader) -> sora_runtime:read_list(fun reward:decode/1, Reader) end)(Reader4),
    {#{
        'id' => Id,
        'name' => Name,
        'monster_ids' => MonsterIds,
        'recommended_power' => RecommendedPower,
        'first_clear_rewards' => FirstClearRewards
    }, Reader5}.
