-module(level_exp).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'level' := integer(),
    'exp' := integer(),
    'unlock_feature' := binary() | undefined
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Level, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Exp, Reader2} = (fun sora_runtime:read_i64/1)(Reader1),
    {UnlockFeature, Reader3} = (fun(Reader) -> sora_runtime:read_optional(fun sora_runtime:read_string/1, Reader) end)(Reader2),
    {#{
        'level' => Level,
        'exp' => Exp,
        'unlock_feature' => UnlockFeature
    }, Reader3}.
