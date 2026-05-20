-module(dialogue).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'speaker_key' := binary(),
    'lines' := [binary()]
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1} = (fun sora_runtime:read_i32/1)(Reader0),
    {SpeakerKey, Reader2} = (fun sora_runtime:read_string/1)(Reader1),
    {Lines, Reader3} = (fun(Reader) -> sora_runtime:read_list(fun sora_runtime:read_string/1, Reader) end)(Reader2),
    {#{
        'id' => Id,
        'speaker_key' => SpeakerKey,
        'lines' => Lines
    }, Reader3}.
