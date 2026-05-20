-module(stage_reward).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'stage_id' := integer(),
    'seq' := integer(),
    'item_id' := integer(),
    'count' := integer()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {StageId, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {Seq, Reader2} = (fun sora_runtime:read_i32/1)(Reader1),
    {ItemId, Reader3} = (fun sora_runtime:read_i32/1)(Reader2),
    {Count, Reader4} = (fun sora_runtime:read_i32/1)(Reader3),
    {#{
        'stage_id' => StageId,
        'seq' => Seq,
        'item_id' => ItemId,
        'count' => Count
    }, Reader4}.
