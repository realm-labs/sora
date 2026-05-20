-module(shop_item).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'shop_id' := integer(),
    'seq' := integer(),
    'item_id' := integer(),
    'price' := resource_cost:t(),
    'daily_limit' := integer() | undefined
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {ShopId, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Seq, Reader2} = (fun sora_runtime:read_i32/1)(Reader1),
    {ItemId, Reader3} = (fun sora_runtime:read_i32/1)(Reader2),
    {Price, Reader4} = (fun resource_cost:decode/1)(Reader3),
    {DailyLimit, Reader5} = (fun(Reader) -> sora_runtime:read_optional(fun sora_runtime:read_i32/1, Reader) end)(Reader4),
    {#{
        'shop_id' => ShopId,
        'seq' => Seq,
        'item_id' => ItemId,
        'price' => Price,
        'daily_limit' => DailyLimit
    }, Reader5}.
