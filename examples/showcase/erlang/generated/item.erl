-module(item).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'name' := binary(),
    'item_type' := item_type:t(),
    'max_stack' := integer(),
    'price' := resource_cost:t(),
    'tags' := [binary()]
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Name, Reader2} = (fun sora_runtime:read_string/1)(Reader1),
    {ItemType, Reader3} = (fun item_type:decode/1)(Reader2),
    {MaxStack, Reader4} = (fun sora_runtime:read_i32/1)(Reader3),
    {Price, Reader5} = (fun resource_cost:decode/1)(Reader4),
    {Tags, Reader6} = (fun(Reader) -> sora_runtime:read_list(fun sora_runtime:read_string/1, Reader) end)(Reader5),
    {#{
        'id' => Id,
        'name' => Name,
        'item_type' => ItemType,
        'max_stack' => MaxStack,
        'price' => Price,
        'tags' => Tags
    }, Reader6}.
