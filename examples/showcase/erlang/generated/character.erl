-module(character).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'name' := binary(),
    'rarity' := rarity:t(),
    'base_level' := integer(),
    'base_skill' := integer(),
    'starter_items' := [integer()],
    'spawn_pos' := vec3:t()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1 } = (fun sora_runtime:read_i32/1)(Reader0),
    {Name, Reader2 } = (fun sora_runtime:read_string/1)(Reader1),
    {Rarity, Reader3 } = (fun rarity:decode/1)(Reader2),
    {BaseLevel, Reader4 } = (fun sora_runtime:read_i32/1)(Reader3),
    {BaseSkill, Reader5 } = (fun sora_runtime:read_i32/1)(Reader4),
    {StarterItems, Reader6 } = (fun(Reader) -> sora_runtime:read_list(fun sora_runtime:read_i32/1, Reader) end)(Reader5),
    {SpawnPos, Reader7 } = (fun vec3:decode/1)(Reader6),
    { #{
        'id' => Id,
        'name' => Name,
        'rarity' => Rarity,
        'base_level' => BaseLevel,
        'base_skill' => BaseSkill,
        'starter_items' => StarterItems,
        'spawn_pos' => SpawnPos
    }, Reader7}.
