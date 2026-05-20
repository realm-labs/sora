-module(buff).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'name' := binary(),
    'duration' := float(),
    'modifiers' := [stat_modifier:t()]
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Name, Reader2} = (fun sora_runtime:read_string/1)(Reader1),
    {Duration, Reader3} = (fun sora_runtime:read_f32/1)(Reader2),
    {Modifiers, Reader4} = (fun(Reader) -> sora_runtime:read_list(fun stat_modifier:decode/1, Reader) end)(Reader3),
    {#{
        'id' => Id,
        'name' => Name,
        'duration' => Duration,
        'modifiers' => Modifiers
    }, Reader4}.
