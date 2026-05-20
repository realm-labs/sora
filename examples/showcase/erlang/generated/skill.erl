-module(skill).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'name' := binary(),
    'element' := element_type:t(),
    'cost' := resource_cost:t(),
    'effect' := skill_effect:t(),
    'required_level' := integer(),
    'required_item' := integer() | undefined,
    'cast_origin' := vec3:t()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Name, Reader2} = (fun sora_runtime:read_string/1)(Reader1),
    {Element, Reader3} = (fun element_type:decode/1)(Reader2),
    {Cost, Reader4} = (fun resource_cost:decode/1)(Reader3),
    {Effect, Reader5} = (fun skill_effect:decode/1)(Reader4),
    {RequiredLevel, Reader6} = (fun sora_runtime:read_i32/1)(Reader5),
    {RequiredItem, Reader7} = (fun(Reader) -> sora_runtime:read_optional(fun sora_runtime:read_i32/1, Reader) end)(Reader6),
    {CastOrigin, Reader8} = (fun vec3:decode/1)(Reader7),
    {#{
        'id' => Id,
        'name' => Name,
        'element' => Element,
        'cost' => Cost,
        'effect' => Effect,
        'required_level' => RequiredLevel,
        'required_item' => RequiredItem,
        'cast_origin' => CastOrigin
    }, Reader8}.
