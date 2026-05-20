-module(mail_template).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'id' := integer(),
    'mail_type' := mail_type:t(),
    'title_key' := binary(),
    'body_key' := binary(),
    'rewards' := [reward:t()]
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {Id, Reader1 } = (fun sora_runtime:read_i32/1)(Reader0),
    {MailType, Reader2 } = (fun mail_type:decode/1)(Reader1),
    {TitleKey, Reader3 } = (fun sora_runtime:read_string/1)(Reader2),
    {BodyKey, Reader4 } = (fun sora_runtime:read_string/1)(Reader3),
    {Rewards, Reader5 } = (fun(Reader) -> sora_runtime:read_list(fun reward:decode/1, Reader) end)(Reader4),
    { #{
        'id' => Id,
        'mail_type' => MailType,
        'title_key' => TitleKey,
        'body_key' => BodyKey,
        'rewards' => Rewards
    }, Reader5}.
