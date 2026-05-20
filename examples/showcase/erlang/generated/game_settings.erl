-module(game_settings).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'version' := binary(),
    'daily_reset_hour' := integer(),
    'starting_gold' := integer(),
    'spawn_pos' := vec3:t(),
    'starter_items' := [integer()]
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Version, Reader1} = (fun sora_runtime:read_string/1)(Reader0),
    {DailyResetHour, Reader2} = (fun sora_runtime:read_i32/1)(Reader1),
    {StartingGold, Reader3} = (fun sora_runtime:read_i32/1)(Reader2),
    {SpawnPos, Reader4} = (fun vec3:decode/1)(Reader3),
    {StarterItems, Reader5} = (fun(Reader) -> sora_runtime:read_list(fun sora_runtime:read_i32/1, Reader) end)(Reader4),
    {#{
        'version' => Version,
        'daily_reset_hour' => DailyResetHour,
        'starting_gold' => StartingGold,
        'spawn_pos' => SpawnPos,
        'starter_items' => StarterItems
    }, Reader5}.
