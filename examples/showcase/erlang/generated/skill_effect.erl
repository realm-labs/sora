-module(skill_effect).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'element' := element_type:t(),
    'power' := integer(),
    'radius' := float()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Element, Reader1} = (fun element_type:decode/1)(Reader0),
    {Power, Reader2} = (fun sora_runtime:read_i32/1)(Reader1),
    {Radius, Reader3} = (fun sora_runtime:read_f32/1)(Reader2),
    {#{
        'element' => Element,
        'power' => Power,
        'radius' => Radius
    }, Reader3}.
