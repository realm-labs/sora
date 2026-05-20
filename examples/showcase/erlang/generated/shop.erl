-module(shop).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'name' := binary(),
    'currency' := resource_kind:t()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1 } = (fun sora_runtime:read_i32/1)(Reader0),
    {Name, Reader2 } = (fun sora_runtime:read_string/1)(Reader1),
    {Currency, Reader3 } = (fun resource_kind:decode/1)(Reader2),
    { #{
        'id' => Id,
        'name' => Name,
        'currency' => Currency
    }, Reader3}.
