-module(vip_level).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'level' := integer(),
    'cost' := resource_cost:t(),
    'perks' := [binary()]
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Level, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Cost, Reader2} = (fun resource_cost:decode/1)(Reader1),
    {Perks, Reader3} = (fun(Reader) -> sora_runtime:read_list(fun sora_runtime:read_string/1, Reader) end)(Reader2),
    {#{
        'level' => Level,
        'cost' => Cost,
        'perks' => Perks
    }, Reader3}.
