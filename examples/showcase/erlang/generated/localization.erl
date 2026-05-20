-module(localization).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'key' := binary(),
    'zh_cn' := binary(),
    'en_us' := binary(),
    'note' := binary() | undefined
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Key, Reader1 } = (fun sora_runtime:read_string/1)(Reader0),
    {ZhCn, Reader2 } = (fun sora_runtime:read_string/1)(Reader1),
    {EnUs, Reader3 } = (fun sora_runtime:read_string/1)(Reader2),
    {Note, Reader4 } = (fun(Reader) -> sora_runtime:read_optional(fun sora_runtime:read_string/1, Reader) end)(Reader3),
    { #{
        'key' => Key,
        'zh_cn' => ZhCn,
        'en_us' => EnUs,
        'note' => Note
    }, Reader4}.
