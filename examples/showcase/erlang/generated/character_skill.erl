-module(character_skill).

-export([decode/1]).
-export_type([t/0]).

-type t() :: #{
    'character_id' := integer(),
    'skill_id' := integer(),
    'unlock_level' := integer()
}.

-spec decode(sora_runtime:reader()) -> {t(), sora_runtime:reader()}.
decode(Reader0) ->
    {CharacterId, Reader1 } = (fun sora_runtime:read_i32/1)(Reader0),
    {SkillId, Reader2 } = (fun sora_runtime:read_i32/1)(Reader1),
    {UnlockLevel, Reader3 } = (fun sora_runtime:read_i32/1)(Reader2),
    { #{
        'character_id' => CharacterId,
        'skill_id' => SkillId,
        'unlock_level' => UnlockLevel
    }, Reader3}.
