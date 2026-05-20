-module(drop_entry).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'group_id' := integer(),
    'seq' := integer(),
    'item_id' := integer(),
    'count' := integer(),
    'weight' := float()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {GroupId, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Seq, Reader2} = (fun sora_runtime:read_i32/1)(Reader1),
    {ItemId, Reader3} = (fun sora_runtime:read_i32/1)(Reader2),
    {Count, Reader4} = (fun sora_runtime:read_i32/1)(Reader3),
    {Weight, Reader5} = (fun sora_runtime:read_f32/1)(Reader4),
    {#{
        'group_id' => GroupId,
        'seq' => Seq,
        'item_id' => ItemId,
        'count' => Count,
        'weight' => Weight
    }, Reader5}.
