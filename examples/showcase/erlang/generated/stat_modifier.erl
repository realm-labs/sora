-module(stat_modifier).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'stat' := stat_type:t(),
    'value' := float(),
    'is_percent' := boolean()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Stat, Reader1 } = (fun stat_type:decode/1)(Reader0),
    {Value, Reader2 } = (fun sora_runtime:read_f32/1)(Reader1),
    {IsPercent, Reader3 } = (fun sora_runtime:read_bool/1)(Reader2),
    { #{
        'stat' => Stat,
        'value' => Value,
        'is_percent' => IsPercent
    }, Reader3}.
