sora-protobufcom.sora.showcase"¿—{"package":"com.sora.showcase","localization":{"locales":["zh_cn","en_us"],"default_locale":"zh_cn","fallback_locale":"en_us","sources":[{"name":"Localization","file":"Core.xlsx","sheet":"Localization","key":"key"}]},"enums":[{"name":"ItemType","scope":{"values":["all"]},"values":[{"id":0,"name":"Weapon"},{"id":1,"name":"Armor"},{"id":2,"name":"Currency"},{"id":3,"name":"Material"},{"id":4,"name":"Consumable"}]},{"name":"ResourceKind","scope":{"values":["all"]},"values":[{"id":0,"name":"Item"},{"id":1,"name":"Gold"},{"id":2,"name":"Diamond"}]},{"name":"ElementType","scope":{"values":["all"]},"values":[{"id":0,"name":"Fire"},{"id":1,"name":"Ice"},{"id":2,"name":"Lightning"},{"id":3,"name":"Physical"}]},{"name":"QuestType","scope":{"values":["all"]},"values":[{"id":0,"name":"Main"},{"id":1,"name":"Side"},{"id":2,"name":"Daily"}]},{"name":"Rarity","scope":{"values":["all"]},"values":[{"id":0,"name":"Common"},{"id":1,"name":"Uncommon"},{"id":2,"name":"Rare"},{"id":3,"name":"Epic"},{"id":4,"name":"Legendary"}]},{"name":"StatType","scope":{"values":["all"]},"values":[{"id":0,"name":"Hp"},{"id":1,"name":"Attack"},{"id":2,"name":"Defense"},{"id":3,"name":"Speed"},{"id":4,"name":"CritRate"}]},{"name":"MailType","scope":{"values":["all"]},"values":[{"id":0,"name":"System"},{"id":1,"name":"Event"},{"id":2,"name":"Compensation"}]}],"structs":[{"name":"ResourceCost","scope":{"values":["all"]},"fields":[{"name":"kind","ty":{"Enum":"ResourceKind"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"id","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"count","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[1,999999],"length":null,"parser":null,"derived_from":null}]},{"name":"Vec3","scope":{"values":["all"]},"fields":[{"name":"x","ty":"F32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"y","ty":"F32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"z","ty":"F32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}]},{"name":"SkillEffect","scope":{"values":["all"]},"fields":[{"name":"element","ty":{"Enum":"ElementType"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"power","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[1,9999],"length":null,"parser":null,"derived_from":null},{"name":"radius","ty":"F32","scope":{"values":["all"]},"key":false,"comment":null,"default":"1.0","range":null,"length":null,"parser":null,"derived_from":null}]},{"name":"Reward","scope":{"values":["all"]},"fields":[{"name":"item_id","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"count","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[1,9999],"length":null,"parser":null,"derived_from":null}]},{"name":"StatModifier","scope":{"values":["all"]},"fields":[{"name":"stat","ty":{"Enum":"StatType"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"value","ty":"F32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"is_percent","ty":"Bool","scope":{"values":["all"]},"key":false,"comment":null,"default":"false","range":null,"length":null,"parser":null,"derived_from":null}]},{"name":"RewardBundle","scope":{"values":["all"]},"fields":[{"name":"cost","ty":{"Struct":"ResourceCost"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{"separator":":"}},"derived_from":null},{"name":"weight","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[1,10000],"length":null,"parser":null,"derived_from":null},{"name":"labels","ty":{"List":"String"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"split","options":{"separator":"+"}},"derived_from":null}]},{"name":"ComplexBudget","scope":{"values":["all"]},"fields":[{"name":"fixed","ty":{"Struct":"ResourceCost"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{"separator":":"}},"derived_from":null},{"name":"random","ty":{"List":{"Struct":"RewardBundle"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple_list","options":{"item_separator":"|","separator":";"}},"derived_from":null},{"name":"limits","ty":{"Map":{"key":"String","value":"I32"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"map","options":{"item_separator":"|","separator":":"}},"derived_from":null}]},{"name":"MaintenanceInfo","scope":{"values":["all"]},"fields":[{"name":"starts_at","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"duration_minutes","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[1,1440],"length":null,"parser":null,"derived_from":null},{"name":"reason","ty":{"Optional":"String"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}]}],"unions":[{"name":"EventCondition","scope":{"values":["all"]},"tag":"type","variants":[{"name":"LevelAtLeast","scope":{"values":["all"]},"fields":[{"name":"level","ty":{"Ref":{"table":"LevelExp","field":"level"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}]},{"name":"QuestCompleted","scope":{"values":["all"]},"fields":[{"name":"quest_id","ty":{"Ref":{"table":"Quest","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}]},{"name":"HasItem","scope":{"values":["all"]},"fields":[{"name":"item_id","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"count","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[1,9999],"length":null,"parser":null,"derived_from":null}]},{"name":"AllConditions","scope":{"values":["all"]},"fields":[{"name":"condition_group_id","ty":{"Ref":{"table":"ComplexConditionGroup","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}]},{"name":"AnyCondition","scope":{"values":["all"]},"fields":[{"name":"condition_group_id","ty":{"Ref":{"table":"ComplexConditionGroup","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}]}]},{"name":"RewardAction","scope":{"values":["all"]},"tag":"type","variants":[{"name":"AddItem","scope":{"values":["all"]},"fields":[{"name":"item_id","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"count","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[1,9999],"length":null,"parser":null,"derived_from":null}]},{"name":"AddBuff","scope":{"values":["all"]},"fields":[{"name":"buff_id","ty":{"Ref":{"table":"Buff","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"duration","ty":"F32","scope":{"values":["all"]},"key":false,"comment":null,"default":"10.0","range":null,"length":null,"parser":null,"derived_from":null}]},{"name":"UnlockStage","scope":{"values":["all"]},"fields":[{"name":"stage_id","ty":{"Ref":{"table":"Stage","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}]},{"name":"SendMail","scope":{"values":["all"]},"fields":[{"name":"mail_id","ty":{"Ref":{"table":"MailTemplate","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}]},{"name":"RunActionGroup","scope":{"values":["all"]},"fields":[{"name":"action_group_id","ty":{"Ref":{"table":"ComplexActionGroup","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}]}]}],"tables":[{"name":"Item","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Core.xlsx","sheet":"Item"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":"Item id","default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":"Display name","default":null,"range":null,"length":[2,32],"parser":null,"derived_from":null},{"name":"item_type","ty":{"Enum":"ItemType"},"scope":{"values":["all"]},"key":false,"comment":"Item category","default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"max_stack","ty":"I32","scope":{"values":["all"]},"key":false,"comment":"Stack limit; blank cells use the default","default":"1","range":[1,9999],"length":null,"parser":null,"derived_from":null},{"name":"price","ty":{"Struct":"ResourceCost"},"scope":{"values":["all"]},"key":false,"comment":"Struct columns: price_kind, price_id, price_count","default":null,"range":null,"length":null,"parser":{"kind":"columns","options":{"prefix":"price_"}},"derived_from":null},{"name":"tags","ty":{"Set":"String"},"scope":{"values":["all"]},"key":false,"comment":"JSON string set","default":"[\"misc\"]","range":null,"length":[1,5],"parser":{"kind":"json","options":{}},"derived_from":null},{"name":"attributes","ty":{"Map":{"key":"String","value":"I32"}},"scope":{"values":["all"]},"key":false,"comment":"Map pairs: key,value|key,value","default":null,"range":null,"length":null,"parser":{"kind":"map","options":{}},"derived_from":null}],"indexes":[{"name":"by_name","fields":["name"],"unique":true},{"name":"by_item_type","fields":["item_type"],"unique":false}]},{"name":"Shop","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Economy.xlsx","sheet":"Shop"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"currency","ty":{"Enum":"ResourceKind"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"ShopItem","scope":{"values":["all"]},"mode":"List","key":null,"source":{"file":"Economy.xlsx","sheet":"ShopItem"},"fields":[{"name":"shop_id","ty":{"Ref":{"table":"Shop","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"seq","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"item_id","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"price","ty":{"Struct":"ResourceCost"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null},{"name":"daily_limit","ty":{"Optional":"I32"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"Recipe","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Economy.xlsx","sheet":"Recipe"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"result_item","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"materials","ty":{"List":{"Struct":"ResourceCost"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[1,4],"parser":{"kind":"tuple_list","options":{}},"derived_from":null}],"indexes":[]},{"name":"GachaPool","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Economy.xlsx","sheet":"GachaPool"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"cost","ty":{"Struct":"ResourceCost"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null}],"indexes":[]},{"name":"GachaItem","scope":{"values":["all"]},"mode":"List","key":null,"source":{"file":"Economy.xlsx","sheet":"GachaItem"},"fields":[{"name":"pool_id","ty":{"Ref":{"table":"GachaPool","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"item_id","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"rarity","ty":{"Enum":"Rarity"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"weight","ty":"F32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"EquipmentSet","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Economy.xlsx","sheet":"EquipmentSet"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"item_ids","ty":{"List":{"Ref":{"table":"Item","field":"id"}}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[2,4],"parser":{"kind":"json","options":{}},"derived_from":null},{"name":"bonus_effect","ty":{"Struct":"SkillEffect"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"Skill","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Battle.xlsx","sheet":"Skill"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[2,32],"parser":null,"derived_from":null},{"name":"element","ty":{"Enum":"ElementType"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"cost","ty":{"Struct":"ResourceCost"},"scope":{"values":["all"]},"key":false,"comment":"Tuple cost, e.g. Gold,0,150","default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null},{"name":"effect","ty":{"Struct":"SkillEffect"},"scope":{"values":["all"]},"key":false,"comment":"JSON object with element/power/radius","default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"required_level","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":"1","range":[1,100],"length":null,"parser":null,"derived_from":null},{"name":"required_item","ty":{"Optional":{"Ref":{"table":"Item","field":"id"}}},"scope":{"values":["all"]},"key":false,"comment":"Optional item requirement","default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"cast_origin","ty":{"Struct":"Vec3"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null}],"indexes":[]},{"name":"Character","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Battle.xlsx","sheet":"Character"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[2,32],"parser":null,"derived_from":null},{"name":"rarity","ty":{"Enum":"Rarity"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"base_level","ty":{"Ref":{"table":"LevelExp","field":"level"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"base_skill","ty":{"Ref":{"table":"Skill","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"starter_items","ty":{"List":{"Ref":{"table":"Item","field":"id"}}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[1,4],"parser":{"kind":"json","options":{}},"derived_from":null},{"name":"spawn_pos","ty":{"Struct":"Vec3"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null}],"indexes":[]},{"name":"CharacterSkill","scope":{"values":["all"]},"mode":"List","key":null,"source":{"file":"Battle.xlsx","sheet":"CharacterSkill"},"fields":[{"name":"character_id","ty":{"Ref":{"table":"Character","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"skill_id","ty":{"Ref":{"table":"Skill","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"unlock_level","ty":{"Ref":{"table":"LevelExp","field":"level"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"Buff","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Battle.xlsx","sheet":"Buff"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"duration","ty":"Duration","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"modifiers","ty":{"List":{"Struct":"StatModifier"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[1,3],"parser":{"kind":"json","options":{}},"derived_from":null}],"indexes":[]},{"name":"DropGroup","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Battle.xlsx","sheet":"DropGroup"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"DropEntry","scope":{"values":["all"]},"mode":"List","key":null,"source":{"file":"Battle.xlsx","sheet":"DropEntry"},"fields":[{"name":"group_id","ty":{"Ref":{"table":"DropGroup","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"seq","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"item_id","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"count","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[1,9999],"length":null,"parser":null,"derived_from":null},{"name":"weight","ty":"F32","scope":{"values":["server"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"Monster","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Battle.xlsx","sheet":"Monster"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"level","ty":{"Ref":{"table":"LevelExp","field":"level"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"element","ty":{"Enum":"ElementType"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"drop_group","ty":{"Ref":{"table":"DropGroup","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"spawn_pos","ty":{"Struct":"Vec3"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null}],"indexes":[]},{"name":"Stage","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Battle.xlsx","sheet":"Stage"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"monster_ids","ty":{"List":{"Ref":{"table":"Monster","field":"id"}}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[1,5],"parser":{"kind":"json","options":{}},"derived_from":null},{"name":"recommended_power","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"first_clear_rewards","ty":{"List":{"Struct":"Reward"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":{"source_table":"StageReward","parent_key":"id","child_key":"stage_id","value_field":null,"order_by":"seq"}}],"indexes":[]},{"name":"StageReward","scope":{"values":["all"]},"mode":"List","key":null,"source":{"file":"Battle.xlsx","sheet":"StageReward"},"fields":[{"name":"stage_id","ty":{"Ref":{"table":"Stage","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"seq","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"item_id","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"count","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"Dungeon","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Battle.xlsx","sheet":"Dungeon"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"stage_ids","ty":{"List":{"Ref":{"table":"Stage","field":"id"}}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[2,6],"parser":{"kind":"json","options":{}},"derived_from":null},{"name":"entry_cost","ty":{"Struct":"ResourceCost"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null}],"indexes":[]},{"name":"Quest","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Quest.xlsx","sheet":"Quest"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"quest_type","ty":{"Enum":"QuestType"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"title","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[4,64],"parser":null,"derived_from":null},{"name":"required_item","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"unlock_skills","ty":{"List":{"Ref":{"table":"Skill","field":"id"}}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[1,3],"parser":{"kind":"json","options":{}},"derived_from":null},{"name":"start_pos","ty":{"Struct":"Vec3"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null},{"name":"rewards","ty":{"List":{"Struct":"Reward"}},"scope":{"values":["all"]},"key":false,"comment":"Materialized from QuestReward child rows","default":null,"range":null,"length":null,"parser":null,"derived_from":{"source_table":"QuestReward","parent_key":"id","child_key":"quest_id","value_field":null,"order_by":"seq"}}],"indexes":[]},{"name":"QuestReward","scope":{"values":["all"]},"mode":"List","key":null,"source":{"file":"Quest.xlsx","sheet":"QuestReward"},"fields":[{"name":"quest_id","ty":{"Ref":{"table":"Quest","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"seq","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"item_id","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"count","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[1,9999],"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"LevelExp","scope":{"values":["all"]},"mode":"Map","key":"level","source":{"file":"Core.xlsx","sheet":"LevelExp"},"fields":[{"name":"level","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":[1,100],"length":null,"parser":null,"derived_from":null},{"name":"exp","ty":"I64","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[0,999999999],"length":null,"parser":null,"derived_from":null},{"name":"unlock_feature","ty":{"Optional":"String"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"Achievement","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Economy.xlsx","sheet":"Achievement"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"title_key","ty":"Text","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"target_count","ty":"I64","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"reward","ty":{"Struct":"ResourceCost"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null}],"indexes":[]},{"name":"VipLevel","scope":{"values":["all"]},"mode":"Map","key":"level","source":{"file":"Economy.xlsx","sheet":"VipLevel"},"fields":[{"name":"level","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"cost","ty":{"Struct":"ResourceCost"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null},{"name":"perks","ty":{"List":"String"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[1,5],"parser":{"kind":"json","options":{}},"derived_from":null}],"indexes":[]},{"name":"GameSettings","scope":{"values":["all"]},"mode":"Singleton","key":null,"source":{"file":"Core.xlsx","sheet":"GameSettings"},"fields":[{"name":"version","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"daily_reset_hour","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[0,23],"length":null,"parser":null,"derived_from":null},{"name":"starting_gold","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":"100","range":[0,999999],"length":null,"parser":null,"derived_from":null},{"name":"spawn_pos","ty":{"Struct":"Vec3"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{}},"derived_from":null},{"name":"starter_items","ty":{"List":{"Ref":{"table":"Item","field":"id"}}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[1,4],"parser":{"kind":"json","options":{}},"derived_from":null},{"name":"gravity","ty":"F64","scope":{"values":["all"]},"key":false,"comment":"Double precision tuning value","default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"daily_bonus_items","ty":{"Array":{"element":{"Ref":{"table":"Item","field":"id"}},"len":3}},"scope":{"values":["all"]},"key":false,"comment":"Fixed-length array parsed from one cell","default":null,"range":null,"length":null,"parser":{"kind":"split","options":{}},"derived_from":null},{"name":"spawn_points","ty":{"Array":{"element":{"Struct":"Vec3"},"len":2}},"scope":{"values":["all"]},"key":false,"comment":"Fixed-length array of structs","default":null,"range":null,"length":null,"parser":{"kind":"tuple_list","options":{}},"derived_from":null},{"name":"maintenance","ty":{"Optional":{"Struct":"MaintenanceInfo"}},"scope":{"values":["all"]},"key":false,"comment":"Optional derived struct copied from a child row","default":null,"range":null,"length":null,"parser":null,"derived_from":{"source_table":"MaintenanceWindow","parent_key":"version","child_key":"version","value_field":null,"order_by":null}}],"indexes":[]},{"name":"MaintenanceWindow","scope":{"values":["all"]},"mode":"List","key":null,"source":{"file":"Core.xlsx","sheet":"MaintenanceWindow"},"fields":[{"name":"version","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"starts_at","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"duration_minutes","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":[1,1440],"length":null,"parser":null,"derived_from":null},{"name":"reason","ty":{"Optional":"String"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"MailTemplate","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Quest.xlsx","sheet":"MailTemplate"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"mail_type","ty":{"Enum":"MailType"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"title_key","ty":"Text","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"body_key","ty":"Text","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"rewards","ty":{"List":{"Struct":"Reward"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":{"source_table":"MailReward","parent_key":"id","child_key":"mail_id","value_field":null,"order_by":"seq"}}],"indexes":[]},{"name":"MailReward","scope":{"values":["all"]},"mode":"List","key":null,"source":{"file":"Quest.xlsx","sheet":"MailReward"},"fields":[{"name":"mail_id","ty":{"Ref":{"table":"MailTemplate","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"seq","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"item_id","ty":{"Ref":{"table":"Item","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"count","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null}],"indexes":[]},{"name":"Dialogue","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Quest.xlsx","sheet":"Dialogue"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"speaker_key","ty":"Text","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"lines","ty":{"List":"String"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[1,5],"parser":{"kind":"json","options":{}},"derived_from":null}],"indexes":[]},{"name":"EventRule","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Quest.xlsx","sheet":"EventRule"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"condition","ty":{"Union":"EventCondition"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"actions","ty":{"List":{"Union":"RewardAction"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":[1,4],"parser":{"kind":"json","options":{}},"derived_from":null}],"indexes":[]},{"name":"ComplexRule","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Complex.xlsx","sheet":"ComplexRule"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"root_condition","ty":{"Union":"EventCondition"},"scope":{"values":["all"]},"key":false,"comment":"Single union value derived from a tagged_columns child row","default":null,"range":null,"length":null,"parser":null,"derived_from":{"source_table":"ComplexRuleCondition","parent_key":"id","child_key":"rule_id","value_field":"value","order_by":null}},{"name":"root_action_group","ty":{"Ref":{"table":"ComplexActionGroup","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"actions","ty":{"List":{"Union":"RewardAction"}},"scope":{"values":["all"]},"key":false,"comment":"Non-JSON list<union<RewardAction>> assembled from child rows","default":null,"range":null,"length":null,"parser":null,"derived_from":{"source_table":"ComplexActionEntry","parent_key":"root_action_group","child_key":"group_id","value_field":"value","order_by":"seq"}},{"name":"budget","ty":{"Struct":"ComplexBudget"},"scope":{"values":["all"]},"key":false,"comment":"Nested tuple, tuple_list, split, and map parsers in one cell","default":null,"range":null,"length":null,"parser":{"kind":"tuple","options":{"separator":","}},"derived_from":null}],"indexes":[]},{"name":"ComplexConditionGroup","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Complex.xlsx","sheet":"ComplexConditionGroup"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"conditions","ty":{"List":{"Union":"EventCondition"}},"scope":{"values":["all"]},"key":false,"comment":"A derived list of union values; each child row is edited without JSON","default":null,"range":null,"length":null,"parser":null,"derived_from":{"source_table":"ComplexConditionGroupEntry","parent_key":"id","child_key":"group_id","value_field":"value","order_by":"seq"}}],"indexes":[]},{"name":"ComplexConditionGroupEntry","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Complex.xlsx","sheet":"ComplexConditionGroupEntry"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"group_id","ty":{"Ref":{"table":"ComplexConditionGroup","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"seq","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"value","ty":{"Union":"EventCondition"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tagged_columns","options":{"prefix":""}},"derived_from":null}],"indexes":[]},{"name":"ComplexRuleCondition","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Complex.xlsx","sheet":"ComplexRuleCondition"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"rule_id","ty":{"Ref":{"table":"ComplexRule","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"value","ty":{"Union":"EventCondition"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tagged_columns","options":{"prefix":""}},"derived_from":null}],"indexes":[]},{"name":"ComplexActionGroup","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Complex.xlsx","sheet":"ComplexActionGroup"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"name","ty":"String","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"actions","ty":{"List":{"Union":"RewardAction"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":{"source_table":"ComplexActionEntry","parent_key":"id","child_key":"group_id","value_field":"value","order_by":"seq"}}],"indexes":[]},{"name":"ComplexActionEntry","scope":{"values":["all"]},"mode":"Map","key":"id","source":{"file":"Complex.xlsx","sheet":"ComplexActionEntry"},"fields":[{"name":"id","ty":"I32","scope":{"values":["all"]},"key":true,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"group_id","ty":{"Ref":{"table":"ComplexActionGroup","field":"id"}},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"seq","ty":"I32","scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":null,"derived_from":null},{"name":"value","ty":{"Union":"RewardAction"},"scope":{"values":["all"]},"key":false,"comment":null,"default":null,"range":null,"length":null,"parser":{"kind":"tagged_columns","options":{"prefix":""}},"derived_from":null}],"indexes":[]}]}*ô…
Item–
1

attributes#*!
*
"tier

*
"power


	
idÈ

	item_type"Weapon

	max_stack

name"
Iron Sword
2
price)2'

countx

id 

kind"Gold

tags*
	"starter
"melee÷
1

attributes#*!
*
"tier

*
"power

	
idÍ

	item_type
"Material

	max_stackÁ

name"Magic Crystal
5
price,2*

count

id 

kind	"Diamond

tags*
"craft
"rareÿ
1

attributes#*!
*
"tier

*
"power

	
id—

	item_type"
Consumable

	max_stack2

name"Health Potion
2
price)2'

count

id 

kind"Gold

tags*
"potion
	"recover‘
1

attributes#*!
*
"tier

*
"power

	
idπ

	item_type
"Currency

	max_stack

name"Training Medal
2
price)2'

count

id 

kind"Gold

tags*
"quest
"token—
1

attributes#*!
*
"tier

*
"power

	
idÎ

	item_type
"Material

	max_stack

name"	Item 1003
2
price)2'

count

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power

	
idÏ

	item_type"
Consumable

	max_stack

name"	Item 1004
2
price)2'

count

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power

	
idÌ

	item_type"Weapon

	max_stack

name"	Item 1005
2
price)2'

count

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power

	
idÓ

	item_type"Armor

	max_stack

name"	Item 1006
2
price)2'

count

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power

	
idÔ

	item_type
"Currency

	max_stack

name"	Item 1007
2
price)2'

count

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power

	
id

	item_type
"Material

	max_stack

name"	Item 1008
2
price)2'

count

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power

	
idÒ

	item_type"
Consumable

	max_stack

name"	Item 1009
2
price)2'

count

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power

	
idÚ

	item_type"Weapon

	max_stack

name"	Item 1010
2
price)2'

count

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power

	
idÛ

	item_type"Armor

	max_stack

name"	Item 1011
2
price)2'

count

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power

	
idÙ

	item_type
"Currency

	max_stack

name"	Item 1012
2
price)2'

count

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power

	
idı

	item_type
"Material

	max_stack

name"	Item 1013
2
price)2'

count

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power

	
idˆ

	item_type"
Consumable

	max_stack

name"	Item 1014
2
price)2'

count

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power

	
id˜

	item_type"Weapon

	max_stack

name"	Item 1015
2
price)2'

count

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power

	
id¯

	item_type"Armor

	max_stack

name"	Item 1016
2
price)2'

count

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power

	
id˘

	item_type
"Currency

	max_stack

name"	Item 1017
2
price)2'

count

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power

	
id˙

	item_type
"Material

	max_stack

name"	Item 1018
2
price)2'

count

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power

	
id˚

	item_type"
Consumable

	max_stack

name"	Item 1019
2
price)2'

count

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power

	
id¸

	item_type"Weapon

	max_stack

name"	Item 1020
2
price)2'

count

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
 
	
id˝

	item_type"Armor

	max_stack

name"	Item 1021
2
price)2'

count

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
!
	
id˛

	item_type
"Currency

	max_stack!

name"	Item 1022
2
price)2'

count 

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
"
	
idˇ

	item_type
"Material

	max_stack"

name"	Item 1023
2
price)2'

count!

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
#
	
idÄ

	item_type"
Consumable

	max_stack#

name"	Item 1024
2
price)2'

count"

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
$
	
idÅ

	item_type"Weapon

	max_stack

name"	Item 1025
2
price)2'

count#

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
%
	
idÇ

	item_type"Armor

	max_stack

name"	Item 1026
2
price)2'

count$

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
&
	
idÉ

	item_type
"Currency

	max_stack&

name"	Item 1027
2
price)2'

count%

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
'
	
idÑ

	item_type
"Material

	max_stack'

name"	Item 1028
2
price)2'

count&

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
(
	
idÖ

	item_type"
Consumable

	max_stack(

name"	Item 1029
2
price)2'

count'

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
)
	
idÜ

	item_type"Weapon

	max_stack

name"	Item 1030
2
price)2'

count(

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
*
	
idá

	item_type"Armor

	max_stack

name"	Item 1031
2
price)2'

count)

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
+
	
idà

	item_type
"Currency

	max_stack+

name"	Item 1032
2
price)2'

count*

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
,
	
idâ

	item_type
"Material

	max_stack,

name"	Item 1033
2
price)2'

count+

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
-
	
idä

	item_type"
Consumable

	max_stack-

name"	Item 1034
2
price)2'

count,

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
.
	
idã

	item_type"Weapon

	max_stack

name"	Item 1035
2
price)2'

count-

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
/
	
idå

	item_type"Armor

	max_stack

name"	Item 1036
2
price)2'

count.

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
0
	
idç

	item_type
"Currency

	max_stack0

name"	Item 1037
2
price)2'

count/

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
1
	
idé

	item_type
"Material

	max_stack1

name"	Item 1038
2
price)2'

count0

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
2
	
idè

	item_type"
Consumable

	max_stack2

name"	Item 1039
2
price)2'

count1

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
3
	
idê

	item_type"Weapon

	max_stack

name"	Item 1040
2
price)2'

count2

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
4
	
idë

	item_type"Armor

	max_stack

name"	Item 1041
2
price)2'

count3

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
5
	
idí

	item_type
"Currency

	max_stack5

name"	Item 1042
2
price)2'

count4

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
6
	
idì

	item_type
"Material

	max_stack6

name"	Item 1043
2
price)2'

count5

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
7
	
idî

	item_type"
Consumable

	max_stack7

name"	Item 1044
2
price)2'

count6

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
8
	
idï

	item_type"Weapon

	max_stack

name"	Item 1045
2
price)2'

count7

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
9
	
idñ

	item_type"Armor

	max_stack

name"	Item 1046
2
price)2'

count8

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
:
	
idó

	item_type
"Currency

	max_stack:

name"	Item 1047
2
price)2'

count9

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
;
	
idò

	item_type
"Material

	max_stack;

name"	Item 1048
2
price)2'

count:

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
<
	
idô

	item_type"
Consumable

	max_stack<

name"	Item 1049
2
price)2'

count;

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
=
	
idö

	item_type"Weapon

	max_stack

name"	Item 1050
2
price)2'

count<

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
>
	
idõ

	item_type"Armor

	max_stack

name"	Item 1051
2
price)2'

count=

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
?
	
idú

	item_type
"Currency

	max_stack?

name"	Item 1052
2
price)2'

count>

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
@
	
idù

	item_type
"Material

	max_stack@

name"	Item 1053
2
price)2'

count?

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
A
	
idû

	item_type"
Consumable

	max_stackA

name"	Item 1054
2
price)2'

count@

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
B
	
idü

	item_type"Weapon

	max_stack

name"	Item 1055
2
price)2'

countA

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
C
	
id†

	item_type"Armor

	max_stack

name"	Item 1056
2
price)2'

countB

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
D
	
id°

	item_type
"Currency

	max_stackD

name"	Item 1057
2
price)2'

countC

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
E
	
id¢

	item_type
"Material

	max_stackE

name"	Item 1058
2
price)2'

countD

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
F
	
id£

	item_type"
Consumable

	max_stackF

name"	Item 1059
2
price)2'

countE

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
G
	
id§

	item_type"Weapon

	max_stack

name"	Item 1060
2
price)2'

countF

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
H
	
id•

	item_type"Armor

	max_stack

name"	Item 1061
2
price)2'

countG

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
I
	
id¶

	item_type
"Currency

	max_stackI

name"	Item 1062
2
price)2'

countH

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
J
	
idß

	item_type
"Material

	max_stackJ

name"	Item 1063
2
price)2'

countI

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
K
	
id®

	item_type"
Consumable

	max_stackK

name"	Item 1064
2
price)2'

countJ

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
L
	
id©

	item_type"Weapon

	max_stack

name"	Item 1065
2
price)2'

countK

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
M
	
id™

	item_type"Armor

	max_stack

name"	Item 1066
2
price)2'

countL

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
N
	
id´

	item_type
"Currency

	max_stackN

name"	Item 1067
2
price)2'

countM

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
O
	
id¨

	item_type
"Material

	max_stackO

name"	Item 1068
2
price)2'

countN

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
P
	
id≠

	item_type"
Consumable

	max_stackP

name"	Item 1069
2
price)2'

countO

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
Q
	
idÆ

	item_type"Weapon

	max_stack

name"	Item 1070
2
price)2'

countP

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
R
	
idØ

	item_type"Armor

	max_stack

name"	Item 1071
2
price)2'

countQ

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
S
	
id∞

	item_type
"Currency

	max_stackS

name"	Item 1072
2
price)2'

countR

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
T
	
id±

	item_type
"Material

	max_stackT

name"	Item 1073
2
price)2'

countS

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
U
	
id≤

	item_type"
Consumable

	max_stackU

name"	Item 1074
2
price)2'

countT

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
V
	
id≥

	item_type"Weapon

	max_stack

name"	Item 1075
2
price)2'

countU

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
W
	
id¥

	item_type"Armor

	max_stack

name"	Item 1076
2
price)2'

countV

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
X
	
idµ

	item_type
"Currency

	max_stackX

name"	Item 1077
2
price)2'

countW

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
Y
	
id∂

	item_type
"Material

	max_stackY

name"	Item 1078
2
price)2'

countX

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
Z
	
id∑

	item_type"
Consumable

	max_stackZ

name"	Item 1079
2
price)2'

countY

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
[
	
id∏

	item_type"Weapon

	max_stack

name"	Item 1080
2
price)2'

countZ

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
\
	
idπ

	item_type"Armor

	max_stack

name"	Item 1081
2
price)2'

count[

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
]
	
id∫

	item_type
"Currency

	max_stack]

name"	Item 1082
2
price)2'

count\

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
^
	
idª

	item_type
"Material

	max_stack^

name"	Item 1083
2
price)2'

count]

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
_
	
idº

	item_type"
Consumable

	max_stack_

name"	Item 1084
2
price)2'

count^

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
`
	
idΩ

	item_type"Weapon

	max_stack

name"	Item 1085
2
price)2'

count_

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
a
	
idæ

	item_type"Armor

	max_stack

name"	Item 1086
2
price)2'

count`

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
b
	
idø

	item_type
"Currency

	max_stackb

name"	Item 1087
2
price)2'

counta

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
c
	
id¿

	item_type
"Material

	max_stackc

name"	Item 1088
2
price)2'

countb

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
d
	
id¡

	item_type"
Consumable

	max_stack

name"	Item 1089
2
price)2'

countc

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
e
	
id¬

	item_type"Weapon

	max_stack

name"	Item 1090
2
price)2'

countd

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
f
	
id√

	item_type"Armor

	max_stack

name"	Item 1091
2
price)2'

counte

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
g
	
idƒ

	item_type
"Currency

	max_stack

name"	Item 1092
2
price)2'

countf

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
h
	
id≈

	item_type
"Material

	max_stack

name"	Item 1093
2
price)2'

countg

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
i
	
id∆

	item_type"
Consumable

	max_stack

name"	Item 1094
2
price)2'

counth

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
j
	
id«

	item_type"Weapon

	max_stack

name"	Item 1095
2
price)2'

counti

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
k
	
id»

	item_type"Armor

	max_stack

name"	Item 1096
2
price)2'

countj

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
l
	
id…

	item_type
"Currency

	max_stack	

name"	Item 1097
2
price)2'

countk

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
m
	
id 

	item_type
"Material

	max_stack


name"	Item 1098
2
price)2'

countl

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
n
	
idÀ

	item_type"
Consumable

	max_stack

name"	Item 1099
2
price)2'

countm

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
o
	
idÃ

	item_type"Weapon

	max_stack

name"	Item 1100
2
price)2'

countn

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
p
	
idÕ

	item_type"Armor

	max_stack

name"	Item 1101
2
price)2'

counto

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
q
	
idŒ

	item_type
"Currency

	max_stack

name"	Item 1102
2
price)2'

countp

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
r
	
idœ

	item_type
"Material

	max_stack

name"	Item 1103
2
price)2'

countq

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
s
	
id–

	item_type"
Consumable

	max_stack

name"	Item 1104
2
price)2'

countr

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
t
	
id—

	item_type"Weapon

	max_stack

name"	Item 1105
2
price)2'

counts

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
u
	
id“

	item_type"Armor

	max_stack

name"	Item 1106
2
price)2'

countt

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
v
	
id”

	item_type
"Currency

	max_stack

name"	Item 1107
2
price)2'

countu

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
w
	
id‘

	item_type
"Material

	max_stack

name"	Item 1108
2
price)2'

countv

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
x
	
id’

	item_type"
Consumable

	max_stack

name"	Item 1109
2
price)2'

countw

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
y
	
id÷

	item_type"Weapon

	max_stack

name"	Item 1110
2
price)2'

countx

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power
z
	
id◊

	item_type"Armor

	max_stack

name"	Item 1111
2
price)2'

county

id 

kind"Gold

tags*
"auto
"armor—
1

attributes#*!
*
"tier

*
"power
{
	
idÿ

	item_type
"Currency

	max_stack

name"	Item 1112
2
price)2'

countz

id 

kind"Gold

tags*
"auto

"currency—
1

attributes#*!
*
"tier

*
"power
|
	
idŸ

	item_type
"Material

	max_stack

name"	Item 1113
2
price)2'

count{

id 

kind"Gold

tags*
"auto

"material’
1

attributes#*!
*
"tier

*
"power
}
	
id⁄

	item_type"
Consumable

	max_stack

name"	Item 1114
2
price)2'

count|

id 

kind"Gold
 
tags*
"auto
"
consumableÕ
1

attributes#*!
*
"tier

*
"power
~
	
id€

	item_type"Weapon

	max_stack

name"	Item 1115
2
price)2'

count}

id 

kind"Gold

tags*
"auto
"weaponÀ
1

attributes#*!
*
"tier

*
"power

	
id‹

	item_type"Armor

	max_stack

name"	Item 1116
2
price)2'

count~

id 

kind"Gold

tags*
"auto
"armor“
2

attributes$*"
*
"tier

*
"power
Ä
	
id›

	item_type
"Currency

	max_stack

name"	Item 1117
2
price)2'

count

id 

kind"Gold

tags*
"auto

"currency”
2

attributes$*"
*
"tier

*
"power
Å
	
idﬁ

	item_type
"Material

	max_stack

name"	Item 1118
3
price*2(

countÄ

id 

kind"Gold

tags*
"auto

"material◊
2

attributes$*"
*
"tier

*
"power
Ç
	
idﬂ

	item_type"
Consumable

	max_stack

name"	Item 1119
3
price*2(

countÅ

id 

kind"Gold
 
tags*
"auto
"
consumableœ
2

attributes$*"
*
"tier

*
"power
É
	
id‡

	item_type"Weapon

	max_stack

name"	Item 1120
3
price*2(

countÇ

id 

kind"Gold

tags*
"auto
"weapon*ã
Shop1

currency"Gold
	
idëN

name"Shop 14

currency	"Diamond
	
idíN

name"Shop 21

currency"Gold
	
idìN

name"Shop 34

currency	"Diamond
	
idîN

name"Shop 41

currency"Gold
	
idïN

name"Shop 5*≤-
ShopItemr

daily_limit8

item_idˆ
2
price)2'

count

id 

kind"Gold
	
seq

shop_idëNr

daily_limit8

item_id˜
2
price)2'

count"

id 

kind"Gold
	
seq

shop_idëNr

daily_limit

item_id¯
2
price)2'

count)

id 

kind"Gold
	
seq

shop_idëNr

daily_limit8

item_id˘
2
price)2'

count0

id 

kind"Gold
	
seq

shop_idëNr

daily_limit8

item_id˙
2
price)2'

count7

id 

kind"Gold
	
seq

shop_idëNr

daily_limit

item_id˚
2
price)2'

count>

id 

kind"Gold
	
seq

shop_idëNr

daily_limit8

item_id¸
2
price)2'

countE

id 

kind"Gold
	
seq

shop_idëNr

daily_limit8

item_id˝
2
price)2'

countL

id 

kind"Gold
	
seq

shop_idëNr

daily_limit

item_id˛
2
price)2'

countS

id 

kind"Gold
	
seq	

shop_idëNr

daily_limit8

item_idˇ
2
price)2'

countZ

id 

kind"Gold
	
seq


shop_idëNr

daily_limit8

item_idÄ
2
price)2'

count

id 

kind"Gold
	
seq

shop_idíNr

daily_limit8

item_idÅ
2
price)2'

count"

id 

kind"Gold
	
seq

shop_idíNr

daily_limit

item_idÇ
2
price)2'

count)

id 

kind"Gold
	
seq

shop_idíNr

daily_limit8

item_idÉ
2
price)2'

count0

id 

kind"Gold
	
seq

shop_idíNr

daily_limit8

item_idÑ
2
price)2'

count7

id 

kind"Gold
	
seq

shop_idíNr

daily_limit

item_idÖ
2
price)2'

count>

id 

kind"Gold
	
seq

shop_idíNr

daily_limit8

item_idÜ
2
price)2'

countE

id 

kind"Gold
	
seq

shop_idíNr

daily_limit8

item_idá
2
price)2'

countL

id 

kind"Gold
	
seq

shop_idíNr

daily_limit

item_idà
2
price)2'

countS

id 

kind"Gold
	
seq	

shop_idíNr

daily_limit8

item_idâ
2
price)2'

countZ

id 

kind"Gold
	
seq


shop_idíNr

daily_limit8

item_idä
2
price)2'

count

id 

kind"Gold
	
seq

shop_idìNr

daily_limit8

item_idã
2
price)2'

count"

id 

kind"Gold
	
seq

shop_idìNr

daily_limit

item_idå
2
price)2'

count)

id 

kind"Gold
	
seq

shop_idìNr

daily_limit8

item_idç
2
price)2'

count0

id 

kind"Gold
	
seq

shop_idìNr

daily_limit8

item_idé
2
price)2'

count7

id 

kind"Gold
	
seq

shop_idìNr

daily_limit

item_idè
2
price)2'

count>

id 

kind"Gold
	
seq

shop_idìNr

daily_limit8

item_idê
2
price)2'

countE

id 

kind"Gold
	
seq

shop_idìNr

daily_limit8

item_idë
2
price)2'

countL

id 

kind"Gold
	
seq

shop_idìNr

daily_limit

item_idí
2
price)2'

countS

id 

kind"Gold
	
seq	

shop_idìNr

daily_limit8

item_idì
2
price)2'

countZ

id 

kind"Gold
	
seq


shop_idìNr

daily_limit8

item_idî
2
price)2'

count

id 

kind"Gold
	
seq

shop_idîNr

daily_limit8

item_idï
2
price)2'

count"

id 

kind"Gold
	
seq

shop_idîNr

daily_limit

item_idñ
2
price)2'

count)

id 

kind"Gold
	
seq

shop_idîNr

daily_limit8

item_idó
2
price)2'

count0

id 

kind"Gold
	
seq

shop_idîNr

daily_limit8

item_idò
2
price)2'

count7

id 

kind"Gold
	
seq

shop_idîNr

daily_limit

item_idô
2
price)2'

count>

id 

kind"Gold
	
seq

shop_idîNr

daily_limit8

item_idö
2
price)2'

countE

id 

kind"Gold
	
seq

shop_idîNr

daily_limit8

item_idõ
2
price)2'

countL

id 

kind"Gold
	
seq

shop_idîNr

daily_limit

item_idú
2
price)2'

countS

id 

kind"Gold
	
seq	

shop_idîNr

daily_limit8

item_idù
2
price)2'

countZ

id 

kind"Gold
	
seq


shop_idîNr

daily_limit8

item_idû
2
price)2'

count

id 

kind"Gold
	
seq

shop_idïNr

daily_limit8

item_idü
2
price)2'

count"

id 

kind"Gold
	
seq

shop_idïNr

daily_limit

item_id†
2
price)2'

count)

id 

kind"Gold
	
seq

shop_idïNr

daily_limit8

item_id°
2
price)2'

count0

id 

kind"Gold
	
seq

shop_idïNr

daily_limit8

item_id¢
2
price)2'

count7

id 

kind"Gold
	
seq

shop_idïNr

daily_limit

item_id£
2
price)2'

count>

id 

kind"Gold
	
seq

shop_idïNr

daily_limit8

item_id§
2
price)2'

countE

id 

kind"Gold
	
seq

shop_idïNr

daily_limit8

item_id•
2
price)2'

countL

id 

kind"Gold
	
seq

shop_idïNr

daily_limit

item_id¶
2
price)2'

countS

id 

kind"Gold
	
seq	

shop_idïNr

daily_limit8

item_idß
2
price)2'

countZ

id 

kind"Gold
	
seq


shop_idïN*– 
Recipeá
	
id˘U
f
	materialsY*W
*2(

count
	
idÌ

kind"Item
)2'

countn

id 

kind"Gold

result_itemÏá
	
id˙U
f
	materialsY*W
*2(

count
	
idÓ

kind"Item
)2'

countx

id 

kind"Gold

result_itemÌà
	
id˚U
g
	materialsZ*X
*2(

count
	
idÔ

kind"Item
*2(

countÇ

id 

kind"Gold

result_itemÓà
	
id¸U
g
	materialsZ*X
*2(

count
	
id

kind"Item
*2(

countå

id 

kind"Gold

result_itemÔà
	
id˝U
g
	materialsZ*X
*2(

count
	
idÒ

kind"Item
*2(

countñ

id 

kind"Gold

result_itemà
	
id˛U
g
	materialsZ*X
*2(

count
	
idÚ

kind"Item
*2(

count†

id 

kind"Gold

result_itemÒà
	
idˇU
g
	materialsZ*X
*2(

count
	
idÛ

kind"Item
*2(

count™

id 

kind"Gold

result_itemÚà
	
idÄV
g
	materialsZ*X
*2(

count
	
idÙ

kind"Item
*2(

count¥

id 

kind"Gold

result_itemÛà
	
idÅV
g
	materialsZ*X
*2(

count
	
idı

kind"Item
*2(

countæ

id 

kind"Gold

result_itemÙà
	
idÇV
g
	materialsZ*X
*2(

count
	
idˆ

kind"Item
*2(

count»

id 

kind"Gold

result_itemıà
	
idÉV
g
	materialsZ*X
*2(

count
	
id˜

kind"Item
*2(

count“

id 

kind"Gold

result_itemˆà
	
idÑV
g
	materialsZ*X
*2(

count
	
id¯

kind"Item
*2(

count‹

id 

kind"Gold

result_item˜à
	
idÖV
g
	materialsZ*X
*2(

count
	
id˘

kind"Item
*2(

countÊ

id 

kind"Gold

result_item¯à
	
idÜV
g
	materialsZ*X
*2(

count
	
id˙

kind"Item
*2(

count

id 

kind"Gold

result_item˘à
	
idáV
g
	materialsZ*X
*2(

count
	
id˚

kind"Item
*2(

count˙

id 

kind"Gold

result_item˙à
	
idàV
g
	materialsZ*X
*2(

count
	
id¸

kind"Item
*2(

countÑ

id 

kind"Gold

result_item˚à
	
idâV
g
	materialsZ*X
*2(

count
	
id˝

kind"Item
*2(

counté

id 

kind"Gold

result_item¸à
	
idäV
g
	materialsZ*X
*2(

count
	
id˛

kind"Item
*2(

countò

id 

kind"Gold

result_item˝à
	
idãV
g
	materialsZ*X
*2(

count
	
idˇ

kind"Item
*2(

count¢

id 

kind"Gold

result_item˛à
	
idåV
g
	materialsZ*X
*2(

count
	
idÄ

kind"Item
*2(

count¨

id 

kind"Gold

result_itemˇà
	
idçV
g
	materialsZ*X
*2(

count
	
idÅ

kind"Item
*2(

count∂

id 

kind"Gold

result_itemÄà
	
idéV
g
	materialsZ*X
*2(

count
	
idÇ

kind"Item
*2(

count¿

id 

kind"Gold

result_itemÅà
	
idèV
g
	materialsZ*X
*2(

count
	
idÉ

kind"Item
*2(

count 

id 

kind"Gold

result_itemÇà
	
idêV
g
	materialsZ*X
*2(

count
	
idÑ

kind"Item
*2(

count‘

id 

kind"Gold

result_itemÉà
	
idëV
g
	materialsZ*X
*2(

count
	
idÖ

kind"Item
*2(

countﬁ

id 

kind"Gold

result_itemÑà
	
idíV
g
	materialsZ*X
*2(

count
	
idÜ

kind"Item
*2(

countË

id 

kind"Gold

result_itemÖà
	
idìV
g
	materialsZ*X
*2(

count
	
idá

kind"Item
*2(

countÚ

id 

kind"Gold

result_itemÜà
	
idîV
g
	materialsZ*X
*2(

count
	
idà

kind"Item
*2(

count¸

id 

kind"Gold

result_itemáà
	
idïV
g
	materialsZ*X
*2(

count
	
idâ

kind"Item
*2(

countÜ

id 

kind"Gold

result_itemàà
	
idñV
g
	materialsZ*X
*2(

count
	
idä

kind"Item
*2(

countê

id 

kind"Gold

result_itemâ*ä
	GachaPoolS
4
cost,2*

count


id 

kind	"Diamond
	
id·]

name"Pool 1S
4
cost,2*

count

id 

kind	"Diamond
	
id‚]

name"Pool 2S
4
cost,2*

count

id 

kind	"Diamond
	
id„]

name"Pool 3*´#
	GachaItemK

item_idä

pool_id·]

rarity
"Uncommon

weight	       @G

item_idã

pool_id·]

rarity"Rare

weight	      @G

item_idå

pool_id·]

rarity"Epic

weight	      @L

item_idç

pool_id·]

rarity"	Legendary

weight	      @I

item_idé

pool_id·]

rarity"Common

weight	      @K

item_idè

pool_id·]

rarity
"Uncommon

weight	      @G

item_idê

pool_id·]

rarity"Rare

weight	       @G

item_idë

pool_id·]

rarity"Epic

weight	      "@L

item_idí

pool_id·]

rarity"	Legendary

weight	      $@I

item_idì

pool_id·]

rarity"Common

weight	      ?K

item_idî

pool_id·]

rarity
"Uncommon

weight	       @G

item_idï

pool_id·]

rarity"Rare

weight	      @G

item_idñ

pool_id·]

rarity"Epic

weight	      @L

item_idó

pool_id·]

rarity"	Legendary

weight	      @I

item_idò

pool_id·]

rarity"Common

weight	      @K

item_idô

pool_id·]

rarity
"Uncommon

weight	      @G

item_idö

pool_id·]

rarity"Rare

weight	       @G

item_idõ

pool_id·]

rarity"Epic

weight	      "@L

item_idú

pool_id·]

rarity"	Legendary

weight	      $@I

item_idù

pool_id·]

rarity"Common

weight	      ?K

item_id®

pool_id‚]

rarity
"Uncommon

weight	       @G

item_id©

pool_id‚]

rarity"Rare

weight	      @G

item_id™

pool_id‚]

rarity"Epic

weight	      @L

item_id´

pool_id‚]

rarity"	Legendary

weight	      @I

item_id¨

pool_id‚]

rarity"Common

weight	      @K

item_id≠

pool_id‚]

rarity
"Uncommon

weight	      @G

item_idÆ

pool_id‚]

rarity"Rare

weight	       @G

item_idØ

pool_id‚]

rarity"Epic

weight	      "@L

item_id∞

pool_id‚]

rarity"	Legendary

weight	      $@I

item_id±

pool_id‚]

rarity"Common

weight	      ?K

item_id≤

pool_id‚]

rarity
"Uncommon

weight	       @G

item_id≥

pool_id‚]

rarity"Rare

weight	      @G

item_id¥

pool_id‚]

rarity"Epic

weight	      @L

item_idµ

pool_id‚]

rarity"	Legendary

weight	      @I

item_id∂

pool_id‚]

rarity"Common

weight	      @K

item_id∑

pool_id‚]

rarity
"Uncommon

weight	      @G

item_id∏

pool_id‚]

rarity"Rare

weight	       @G

item_idπ

pool_id‚]

rarity"Epic

weight	      "@L

item_id∫

pool_id‚]

rarity"	Legendary

weight	      $@I

item_idª

pool_id‚]

rarity"Common

weight	      ?K

item_id∆

pool_id„]

rarity
"Uncommon

weight	       @G

item_id«

pool_id„]

rarity"Rare

weight	      @G

item_id»

pool_id„]

rarity"Epic

weight	      @L

item_id…

pool_id„]

rarity"	Legendary

weight	      @I

item_id 

pool_id„]

rarity"Common

weight	      @K

item_idÀ

pool_id„]

rarity
"Uncommon

weight	      @G

item_idÃ

pool_id„]

rarity"Rare

weight	       @G

item_idÕ

pool_id„]

rarity"Epic

weight	      "@L

item_idŒ

pool_id„]

rarity"	Legendary

weight	      $@I

item_idœ

pool_id„]

rarity"Common

weight	      ?K

item_id–

pool_id„]

rarity
"Uncommon

weight	       @G

item_id—

pool_id„]

rarity"Rare

weight	      @G

item_id“

pool_id„]

rarity"Epic

weight	      @L

item_id”

pool_id„]

rarity"	Legendary

weight	      @I

item_id‘

pool_id„]

rarity"Common

weight	      @K

item_id’

pool_id„]

rarity
"Uncommon

weight	      @G

item_id÷

pool_id„]

rarity"Rare

weight	       @G

item_id◊

pool_id„]

rarity"Epic

weight	      "@L

item_idÿ

pool_id„]

rarity"	Legendary

weight	      $@I

item_idŸ

pool_id„]

rarity"Common

weight	      ?*È

EquipmentSetÉ
F
bonus_effect624

element"Ice

power3

radius	      ?
	
id…e

item_ids*
Ï
Ì
Ó

name"Set 1â
L
bonus_effect<2:

element"	Lightning

power4

radius	      ?
	
id e

item_ids*
Ì
Ó
Ô

name"Set 2à
K
bonus_effect;29

element
"Physical

power5

radius	      ?
	
idÀe

item_ids*
Ó
Ô


name"Set 3Ñ
G
bonus_effect725

element"Fire

power6

radius	      ?
	
idÃe

item_ids*
Ô

Ò

name"Set 4É
F
bonus_effect624

element"Ice

power7

radius	      ?
	
idÕe

item_ids*

Ò
Ú

name"Set 5â
L
bonus_effect<2:

element"	Lightning

power8

radius	      ?
	
idŒe

item_ids*
Ò
Ú
Û

name"Set 6à
K
bonus_effect;29

element
"Physical

power9

radius	      ?
	
idœe

item_ids*
Ú
Û
Ù

name"Set 7Ñ
G
bonus_effect725

element"Fire

power:

radius	      ?
	
id–e

item_ids*
Û
Ù
ı

name"Set 8É
F
bonus_effect624

element"Ice

power;

radius	      ?
	
id—e

item_ids*
Ù
ı
ˆ

name"Set 9ä
L
bonus_effect<2:

element"	Lightning

power<

radius	      ?
	
id“e

item_ids*
ı
ˆ
˜

name"Set 10*˘C
Skillö
A
cast_origin220

x	        

y	333333Û?

z	        
2
cost*2(

countñ

id 

kind"Gold
A
effect725

element"Fire

powerx

radius	      @

element"Fire

ide

name"Flame Slash

required_itemÈ

required_levelï
A
cast_origin220

x	        

y	      ¯?

z	      @
2
cost*2(

count
	
idÍ

kind"Item
@
effect624

element"Ice

power_

radius	      ?

element"Ice

idf

name"	Ice Lance

required_item8

required_level©
A
cast_origin220

x	      @

y	      @

z	      @
2
cost*2(

countµ

id 

kind"Gold
F
effect<2:

element
"Physical

power∑

radius	      @

element
"Physical

idg

name"Physical Skill 103

required_item8

required_levelñ
A
cast_origin220

x	      @

y	      ?

z	      @
2
cost*2(

count∏

id 

kind"Gold
;
effect12/

element"Fire

power∏

radius

element"Fire

idh

name"Fire Skill 104

required_item8

required_levelî
A
cast_origin220

x	        

y	       @

z	        
2
cost*2(

countª

id 

kind"Gold
:
effect02.

element"Ice

powerπ

radius

element"Ice

idi

name"Ice Skill 105

required_itemÍ

required_level¨
A
cast_origin220

x	      ?

y	      @

z	      ?
2
cost*2(

countæ

id 

kind"Gold
G
effect=2;

element"	Lightning

power∫

radius	      ¯?

element"	Lightning

idj

name"Lightning Skill 106

required_item8

required_level¢
A
cast_origin220

x	       @

y	      @

z	       @
2
cost*2(

count¡

id 

kind"Gold
?
effect523

element
"Physical

powerª

radius

element
"Physical

idk

name"Physical Skill 107

required_item8

required_levelû
A
cast_origin220

x	      @

y	      ?

z	      @
2
cost*2(

countƒ

id 

kind"Gold
B
effect826

element"Fire

powerº

radius	      @

element"Fire

idl

name"Fire Skill 108

required_itemÍ

required_level	ì
A
cast_origin220

x	      @

y	       @

z	      @
2
cost*2(

count«

id 

kind"Gold
:
effect02.

element"Ice

powerΩ

radius

element"Ice

idm

name"Ice Skill 109

required_item8

required_level
•
A
cast_origin220

x	        

y	      @

z	      @
2
cost*2(

count 

id 

kind"Gold
@
effect624

element"	Lightning

poweræ

radius

element"	Lightning

idn

name"Lightning Skill 110

required_item8

required_level™
A
cast_origin220

x	      ?

y	      @

z	      @
2
cost*2(

countÕ

id 

kind"Gold
F
effect<2:

element
"Physical

powerø

radius	      ¯?

element
"Physical

ido

name"Physical Skill 111

required_itemÍ

required_levelñ
A
cast_origin220

x	       @

y	      ?

z	        
2
cost*2(

count–

id 

kind"Gold
;
effect12/

element"Fire

power¿

radius

element"Fire

idp

name"Fire Skill 112

required_item8

required_levelö
A
cast_origin220

x	      @

y	       @

z	      ?
2
cost*2(

count”

id 

kind"Gold
A
effect725

element"Ice

power¡

radius	      @

element"Ice

idq

name"Ice Skill 113

required_item8

required_level¶
A
cast_origin220

x	      @

y	      @

z	       @
2
cost*2(

count÷

id 

kind"Gold
@
effect624

element"	Lightning

power¬

radius

element"	Lightning

idr

name"Lightning Skill 114

required_itemÍ

required_level¢
A
cast_origin220

x	        

y	      @

z	      @
2
cost*2(

countŸ

id 

kind"Gold
?
effect523

element
"Physical

power√

radius

element
"Physical

ids

name"Physical Skill 115

required_item8

required_levelù
A
cast_origin220

x	      ?

y	      ?

z	      @
2
cost*2(

count‹

id 

kind"Gold
B
effect826

element"Fire

powerƒ

radius	      ¯?

element"Fire

idt

name"Fire Skill 116

required_item8

required_levelî
A
cast_origin220

x	       @

y	       @

z	      @
2
cost*2(

countﬂ

id 

kind"Gold
:
effect02.

element"Ice

power≈

radius

element"Ice

idu

name"Ice Skill 117

required_itemÍ

required_level¨
A
cast_origin220

x	      @

y	      @

z	      @
2
cost*2(

count‚

id 

kind"Gold
G
effect=2;

element"	Lightning

power∆

radius	      @

element"	Lightning

idv

name"Lightning Skill 118

required_item8

required_level¢
A
cast_origin220

x	      @

y	      @

z	        
2
cost*2(

countÂ

id 

kind"Gold
?
effect523

element
"Physical

power«

radius

element
"Physical

idw

name"Physical Skill 119

required_item8

required_leveló
A
cast_origin220

x	        

y	      ?

z	      ?
2
cost*2(

countË

id 

kind"Gold
;
effect12/

element"Fire

power»

radius

element"Fire

idx

name"Fire Skill 120

required_itemÍ

required_levelö
A
cast_origin220

x	      ?

y	       @

z	       @
2
cost*2(

countÎ

id 

kind"Gold
A
effect725

element"Ice

power…

radius	      ¯?

element"Ice

idy

name"Ice Skill 121

required_item8

required_level•
A
cast_origin220

x	       @

y	      @

z	      @
2
cost*2(

countÓ

id 

kind"Gold
@
effect624

element"	Lightning

power 

radius

element"	Lightning

idz

name"Lightning Skill 122

required_item8

required_level™
A
cast_origin220

x	      @

y	      @

z	      @
2
cost*2(

countÒ

id 

kind"Gold
F
effect<2:

element
"Physical

powerÀ

radius	      @

element
"Physical

id{

name"Physical Skill 123

required_itemÍ

required_levelñ
A
cast_origin220

x	      @

y	      ?

z	      @
2
cost*2(

countÙ

id 

kind"Gold
;
effect12/

element"Fire

powerÃ

radius

element"Fire

id|

name"Fire Skill 124

required_item8

required_levelì
A
cast_origin220

x	        

y	       @

z	      @
2
cost*2(

count˜

id 

kind"Gold
:
effect02.

element"Ice

powerÕ

radius

element"Ice

id}

name"Ice Skill 125

required_item8

required_level≠
A
cast_origin220

x	      ?

y	      @

z	        
2
cost*2(

count˙

id 

kind"Gold
G
effect=2;

element"	Lightning

powerŒ

radius	      ¯?

element"	Lightning

id~

name"Lightning Skill 126

required_itemÍ

required_level¢
A
cast_origin220

x	       @

y	      @

z	      ?
2
cost*2(

count˝

id 

kind"Gold
?
effect523

element
"Physical

powerœ

radius

element
"Physical

id

name"Physical Skill 127

required_item8

required_levelû
A
cast_origin220

x	      @

y	      ?

z	       @
2
cost*2(

countÄ

id 

kind"Gold
B
effect826

element"Fire

power–

radius	      @

element"Fire
	
idÄ

name"Fire Skill 128

required_item8

required_levelï
A
cast_origin220

x	      @

y	       @

z	      @
2
cost*2(

countÉ

id 

kind"Gold
:
effect02.

element"Ice

power—

radius

element"Ice
	
idÅ

name"Ice Skill 129

required_itemÍ

required_level¶
A
cast_origin220

x	        

y	      @

z	      @
2
cost*2(

countÜ

id 

kind"Gold
@
effect624

element"	Lightning

power“

radius

element"	Lightning
	
idÇ

name"Lightning Skill 130

required_item8

required_level*í
	Characterø


base_level


base_skillp
	
id°

name"	Hero 4001

rarity
"Uncommon
?
	spawn_pos220

x	      ?

y	        

z	      @
"
starter_items*
È
÷
◊ª


base_level


base_skillq
	
id¢

name"	Hero 4002

rarity"Rare
?
	spawn_pos220

x	       @

y	        

z	        
"
starter_items*
È
◊
ÿª


base_level


base_skillr
	
id£

name"	Hero 4003

rarity"Epic
?
	spawn_pos220

x	      @

y	        

z	      ?
"
starter_items*
È
ÿ
Ÿ¿


base_level


base_skills
	
id§

name"	Hero 4004

rarity"	Legendary
?
	spawn_pos220

x	      @

y	        

z	       @
"
starter_items*
È
Ÿ
⁄Ω


base_level


base_skillt
	
id•

name"	Hero 4005

rarity"Common
?
	spawn_pos220

x	      @

y	        

z	      @
"
starter_items*
È
⁄
€ø


base_level


base_skillu
	
id¶

name"	Hero 4006

rarity
"Uncommon
?
	spawn_pos220

x	      @

y	        

z	      @
"
starter_items*
È
€
‹ª


base_level


base_skillv
	
idß

name"	Hero 4007

rarity"Rare
?
	spawn_pos220

x	      @

y	        

z	      @
"
starter_items*
È
‹
›ª


base_level	


base_skillw
	
id®

name"	Hero 4008

rarity"Epic
?
	spawn_pos220

x	        

y	        

z	        
"
starter_items*
È
›
ﬁ¿


base_level



base_skillx
	
id©

name"	Hero 4009

rarity"	Legendary
?
	spawn_pos220

x	      ?

y	        

z	      ?
"
starter_items*
È
ﬁ
ﬂΩ


base_level


base_skilly
	
id™

name"	Hero 4010

rarity"Common
?
	spawn_pos220

x	       @

y	        

z	       @
"
starter_items*
È
ﬂ
‡ø


base_level


base_skillz
	
id´

name"	Hero 4011

rarity
"Uncommon
?
	spawn_pos220

x	      @

y	        

z	      @
"
starter_items*
È
‡
Îª


base_level


base_skill{
	
id¨

name"	Hero 4012

rarity"Rare
?
	spawn_pos220

x	      @

y	        

z	      @
"
starter_items*
È
Î
Ïª


base_level


base_skill|
	
id≠

name"	Hero 4013

rarity"Epic
?
	spawn_pos220

x	      @

y	        

z	      @
"
starter_items*
È
Ï
Ì¿


base_level


base_skill}
	
idÆ

name"	Hero 4014

rarity"	Legendary
?
	spawn_pos220

x	      @

y	        

z	        
"
starter_items*
È
Ì
ÓΩ


base_level


base_skill~
	
idØ

name"	Hero 4015

rarity"Common
?
	spawn_pos220

x	      @

y	        

z	      ?
"
starter_items*
È
Ó
Ôø


base_level


base_skill
	
id∞

name"	Hero 4016

rarity
"Uncommon
?
	spawn_pos220

x	        

y	        

z	       @
"
starter_items*
È
Ô
º


base_level


base_skillÄ
	
id±

name"	Hero 4017

rarity"Rare
?
	spawn_pos220

x	      ?

y	        

z	      @
"
starter_items*
È

Òº


base_level


base_skillÅ
	
id≤

name"	Hero 4018

rarity"Epic
?
	spawn_pos220

x	       @

y	        

z	      @
"
starter_items*
È
Ò
Ú¡


base_level


base_skillÇ
	
id≥

name"	Hero 4019

rarity"	Legendary
?
	spawn_pos220

x	      @

y	        

z	      @
"
starter_items*
È
Ú
ÛΩ


base_level


base_skille
	
id¥

name"	Hero 4020

rarity"Common
?
	spawn_pos220

x	      @

y	        

z	        
"
starter_items*
È
Û
Ù*Ì
CharacterSkill9

character_id°

skill_idp

unlock_level9

character_id°

skill_idq

unlock_level9

character_id°

skill_idr

unlock_level9

character_id¢

skill_idq

unlock_level9

character_id¢

skill_idr

unlock_level9

character_id¢

skill_ids

unlock_level9

character_id£

skill_idr

unlock_level9

character_id£

skill_ids

unlock_level9

character_id£

skill_idt

unlock_level9

character_id§

skill_ids

unlock_level9

character_id§

skill_idt

unlock_level9

character_id§

skill_idu

unlock_level9

character_id•

skill_idt

unlock_level9

character_id•

skill_idu

unlock_level9

character_id•

skill_idv

unlock_level9

character_id¶

skill_idu

unlock_level9

character_id¶

skill_idv

unlock_level9

character_id¶

skill_idw

unlock_level9

character_idß

skill_idv

unlock_level9

character_idß

skill_idw

unlock_level9

character_idß

skill_idx

unlock_level9

character_id®

skill_idw

unlock_level9

character_id®

skill_idx

unlock_level9

character_id®

skill_idy

unlock_level9

character_id©

skill_idx

unlock_level9

character_id©

skill_idy

unlock_level9

character_id©

skill_idz

unlock_level9

character_id™

skill_idy

unlock_level9

character_id™

skill_idz

unlock_level9

character_id™

skill_id{

unlock_level9

character_id´

skill_idz

unlock_level9

character_id´

skill_id{

unlock_level9

character_id´

skill_id|

unlock_level9

character_id¨

skill_id{

unlock_level9

character_id¨

skill_id|

unlock_level9

character_id¨

skill_id}

unlock_level9

character_id≠

skill_id|

unlock_level9

character_id≠

skill_id}

unlock_level9

character_id≠

skill_id~

unlock_level9

character_idÆ

skill_id}

unlock_level9

character_idÆ

skill_id~

unlock_level9

character_idÆ

skill_id

unlock_level9

character_idØ

skill_id~

unlock_level9

character_idØ

skill_id

unlock_level:

character_idØ

skill_idÄ

unlock_level9

character_id∞

skill_id

unlock_level:

character_id∞

skill_idÄ

unlock_level:

character_id∞

skill_idÅ

unlock_level:

character_id±

skill_idÄ

unlock_level:

character_id±

skill_idÅ

unlock_level:

character_id±

skill_idÇ

unlock_level:

character_id≤

skill_idÅ

unlock_level:

character_id≤

skill_idÇ

unlock_level9

character_id≤

skill_ide

unlock_level:

character_id≥

skill_idÇ

unlock_level9

character_id≥

skill_ide

unlock_level9

character_id≥

skill_idf

unlock_level9

character_id¥

skill_ide

unlock_level9

character_id¥

skill_idf

unlock_level9

character_id¥

skill_idg

unlock_level*Ú
Buffw

duration†
	
idÒ.
D
	modifiers7*5
321


is_percent 

stat"Attack

value

name"	Buff 6001x

durationà'
	
idÚ.
E
	modifiers8*6
422


is_percent

stat	"Defense

value

name"	Buff 6002v

duration.
	
idÛ.
C
	modifiers6*4
220


is_percent 

stat"Speed

value

name"	Buff 6003y

durationÿ6
	
idÙ.
F
	modifiers9*7
523


is_percent

stat
"CritRate

value	

name"	Buff 6004s

duration¿>
	
idı.
@
	modifiers3*1
/2-


is_percent 

stat"Hp

value


name"	Buff 6005w

duration®F
	
idˆ.
D
	modifiers7*5
321


is_percent

stat"Attack

value

name"	Buff 6006x

durationêN
	
id˜.
E
	modifiers8*6
422


is_percent 

stat	"Defense

value

name"	Buff 6007v

duration∏
	
id¯.
C
	modifiers6*4
220


is_percent

stat"Speed

value

name"	Buff 6008y

duration†
	
id˘.
F
	modifiers9*7
523


is_percent 

stat
"CritRate

value

name"	Buff 6009s

durationà'
	
id˙.
@
	modifiers3*1
/2-


is_percent

stat"Hp

value

name"	Buff 6010w

duration.
	
id˚.
D
	modifiers7*5
321


is_percent 

stat"Attack

value

name"	Buff 6011x

durationÿ6
	
id¸.
E
	modifiers8*6
422


is_percent

stat	"Defense

value

name"	Buff 6012v

duration¿>
	
id˝.
C
	modifiers6*4
220


is_percent 

stat"Speed

value

name"	Buff 6013y

duration®F
	
id˛.
F
	modifiers9*7
523


is_percent

stat
"CritRate

value

name"	Buff 6014s

durationêN
	
idˇ.
@
	modifiers3*1
/2-


is_percent 

stat"Hp

value

name"	Buff 6015w

duration∏
	
idÄ/
D
	modifiers7*5
321


is_percent

stat"Attack

value

name"	Buff 6016x

duration†
	
idÅ/
E
	modifiers8*6
422


is_percent 

stat	"Defense

value

name"	Buff 6017v

durationà'
	
idÇ/
C
	modifiers6*4
220


is_percent

stat"Speed

value

name"	Buff 6018y

duration.
	
idÉ/
F
	modifiers9*7
523


is_percent 

stat
"CritRate

value

name"	Buff 6019s

durationÿ6
	
idÑ/
@
	modifiers3*1
/2-


is_percent

stat"Hp

value

name"	Buff 6020*´
	DropGroup&
	
idŸ6

name"Drop Group 7001&
	
id⁄6

name"Drop Group 7002&
	
id€6

name"Drop Group 7003&
	
id‹6

name"Drop Group 7004&
	
id›6

name"Drop Group 7005&
	
idﬁ6

name"Drop Group 7006&
	
idﬂ6

name"Drop Group 7007&
	
id‡6

name"Drop Group 7008&
	
id·6

name"Drop Group 7009&
	
id‚6

name"Drop Group 7010&
	
id„6

name"Drop Group 7011&
	
id‰6

name"Drop Group 7012&
	
idÂ6

name"Drop Group 7013&
	
idÊ6

name"Drop Group 7014&
	
idÁ6

name"Drop Group 7015&
	
idË6

name"Drop Group 7016&
	
idÈ6

name"Drop Group 7017&
	
idÍ6

name"Drop Group 7018&
	
idÎ6

name"Drop Group 7019&
	
idÏ6

name"Drop Group 7020*À%
	DropEntryN

count

group_idŸ6

item_idì
	
seq

weight	      .@N

count

group_idŸ6

item_idî
	
seq

weight	      4@N

count

group_idŸ6

item_idï
	
seq

weight	      9@N

count

group_id⁄6

item_idî
	
seq

weight	      .@N

count

group_id⁄6

item_idï
	
seq

weight	      4@N

count

group_id⁄6

item_idñ
	
seq

weight	      9@N

count

group_id€6

item_idï
	
seq

weight	      .@N

count

group_id€6

item_idñ
	
seq

weight	      4@N

count

group_id€6

item_idó
	
seq

weight	      9@N

count

group_id‹6

item_idñ
	
seq

weight	      .@N

count

group_id‹6

item_idó
	
seq

weight	      4@N

count

group_id‹6

item_idò
	
seq

weight	      9@N

count

group_id›6

item_idó
	
seq

weight	      .@N

count

group_id›6

item_idò
	
seq

weight	      4@N

count

group_id›6

item_idô
	
seq

weight	      9@N

count

group_idﬁ6

item_idò
	
seq

weight	      .@N

count

group_idﬁ6

item_idô
	
seq

weight	      4@N

count

group_idﬁ6

item_idö
	
seq

weight	      9@N

count

group_idﬂ6

item_idô
	
seq

weight	      .@N

count

group_idﬂ6

item_idö
	
seq

weight	      4@N

count

group_idﬂ6

item_idõ
	
seq

weight	      9@N

count

group_id‡6

item_idö
	
seq

weight	      .@N

count

group_id‡6

item_idõ
	
seq

weight	      4@N

count

group_id‡6

item_idú
	
seq

weight	      9@N

count

group_id·6

item_idõ
	
seq

weight	      .@N

count

group_id·6

item_idú
	
seq

weight	      4@N

count

group_id·6

item_idù
	
seq

weight	      9@N

count

group_id‚6

item_idú
	
seq

weight	      .@N

count

group_id‚6

item_idù
	
seq

weight	      4@N

count

group_id‚6

item_idû
	
seq

weight	      9@N

count

group_id„6

item_idù
	
seq

weight	      .@N

count

group_id„6

item_idû
	
seq

weight	      4@N

count

group_id„6

item_idü
	
seq

weight	      9@N

count

group_id‰6

item_idû
	
seq

weight	      .@N

count

group_id‰6

item_idü
	
seq

weight	      4@N

count

group_id‰6

item_id†
	
seq

weight	      9@N

count

group_idÂ6

item_idü
	
seq

weight	      .@N

count

group_idÂ6

item_id†
	
seq

weight	      4@N

count

group_idÂ6

item_id°
	
seq

weight	      9@N

count

group_idÊ6

item_id†
	
seq

weight	      .@N

count

group_idÊ6

item_id°
	
seq

weight	      4@N

count

group_idÊ6

item_id¢
	
seq

weight	      9@N

count

group_idÁ6

item_id°
	
seq

weight	      .@N

count

group_idÁ6

item_id¢
	
seq

weight	      4@N

count

group_idÁ6

item_id£
	
seq

weight	      9@N

count

group_idË6

item_id¢
	
seq

weight	      .@N

count

group_idË6

item_id£
	
seq

weight	      4@N

count

group_idË6

item_id§
	
seq

weight	      9@N

count

group_idÈ6

item_id£
	
seq

weight	      .@N

count

group_idÈ6

item_id§
	
seq

weight	      4@N

count

group_idÈ6

item_id•
	
seq

weight	      9@N

count

group_idÍ6

item_id§
	
seq

weight	      .@N

count

group_idÍ6

item_id•
	
seq

weight	      4@N

count

group_idÍ6

item_id¶
	
seq

weight	      9@N

count

group_idÎ6

item_id•
	
seq

weight	      .@N

count

group_idÎ6

item_id¶
	
seq

weight	      4@N

count

group_idÎ6

item_idß
	
seq

weight	      9@N

count

group_idÏ6

item_id¶
	
seq

weight	      .@N

count

group_idÏ6

item_idß
	
seq

weight	      4@N

count

group_idÏ6

item_id®
	
seq

weight	      9@*…a
Monsterñ


drop_group⁄6

element"Ice
	
id¡>

level

name"Monster 8001
?
	spawn_pos220

x	      ?

y	        

z	      @ú


drop_group€6

element"	Lightning
	
id¬>

level

name"Monster 8002
?
	spawn_pos220

x	       @

y	        

z	      @õ


drop_group‹6

element
"Physical
	
id√>

level

name"Monster 8003
?
	spawn_pos220

x	      @

y	        

z	       @ó


drop_group›6

element"Fire
	
idƒ>

level

name"Monster 8004
?
	spawn_pos220

x	      @

y	        

z	      "@ñ


drop_groupﬁ6

element"Ice
	
id≈>

level

name"Monster 8005
?
	spawn_pos220

x	      @

y	        

z	      $@ú


drop_groupﬂ6

element"	Lightning
	
id∆>

level

name"Monster 8006
?
	spawn_pos220

x	      @

y	        

z	      &@õ


drop_group‡6

element
"Physical
	
id«>

level

name"Monster 8007
?
	spawn_pos220

x	      @

y	        

z	      (@ó


drop_group·6

element"Fire
	
id»>

level	

name"Monster 8008
?
	spawn_pos220

x	       @

y	        

z	      *@ñ


drop_group‚6

element"Ice
	
id…>

level


name"Monster 8009
?
	spawn_pos220

x	      "@

y	        

z	      ,@ú


drop_group„6

element"	Lightning
	
id >

level

name"Monster 8010
?
	spawn_pos220

x	      $@

y	        

z	        õ


drop_group‰6

element
"Physical
	
idÀ>

level

name"Monster 8011
?
	spawn_pos220

x	      &@

y	        

z	      ?ó


drop_groupÂ6

element"Fire
	
idÃ>

level

name"Monster 8012
?
	spawn_pos220

x	      (@

y	        

z	       @ñ


drop_groupÊ6

element"Ice
	
idÕ>

level

name"Monster 8013
?
	spawn_pos220

x	      *@

y	        

z	      @ú


drop_groupÁ6

element"	Lightning
	
idŒ>

level

name"Monster 8014
?
	spawn_pos220

x	      ,@

y	        

z	      @õ


drop_groupË6

element
"Physical
	
idœ>

level

name"Monster 8015
?
	spawn_pos220

x	      .@

y	        

z	      @ó


drop_groupÈ6

element"Fire
	
id–>

level

name"Monster 8016
?
	spawn_pos220

x	      0@

y	        

z	      @ñ


drop_groupÍ6

element"Ice
	
id—>

level

name"Monster 8017
?
	spawn_pos220

x	      1@

y	        

z	      @ú


drop_groupÎ6

element"	Lightning
	
id“>

level

name"Monster 8018
?
	spawn_pos220

x	      2@

y	        

z	       @õ


drop_groupÏ6

element
"Physical
	
id”>

level

name"Monster 8019
?
	spawn_pos220

x	      3@

y	        

z	      "@ó


drop_groupŸ6

element"Fire
	
id‘>

level

name"Monster 8020
?
	spawn_pos220

x	        

y	        

z	      $@ñ


drop_group⁄6

element"Ice
	
id’>

level

name"Monster 8021
?
	spawn_pos220

x	      ?

y	        

z	      &@ú


drop_group€6

element"	Lightning
	
id÷>

level

name"Monster 8022
?
	spawn_pos220

x	       @

y	        

z	      (@õ


drop_group‹6

element
"Physical
	
id◊>

level

name"Monster 8023
?
	spawn_pos220

x	      @

y	        

z	      *@ó


drop_group›6

element"Fire
	
idÿ>

level

name"Monster 8024
?
	spawn_pos220

x	      @

y	        

z	      ,@ñ


drop_groupﬁ6

element"Ice
	
idŸ>

level

name"Monster 8025
?
	spawn_pos220

x	      @

y	        

z	        ú


drop_groupﬂ6

element"	Lightning
	
id⁄>

level

name"Monster 8026
?
	spawn_pos220

x	      @

y	        

z	      ?õ


drop_group‡6

element
"Physical
	
id€>

level

name"Monster 8027
?
	spawn_pos220

x	      @

y	        

z	       @ó


drop_group·6

element"Fire
	
id‹>

level

name"Monster 8028
?
	spawn_pos220

x	       @

y	        

z	      @ñ


drop_group‚6

element"Ice
	
id›>

level

name"Monster 8029
?
	spawn_pos220

x	      "@

y	        

z	      @ú


drop_group„6

element"	Lightning
	
idﬁ>

level

name"Monster 8030
?
	spawn_pos220

x	      $@

y	        

z	      @õ


drop_group‰6

element
"Physical
	
idﬂ>

level 

name"Monster 8031
?
	spawn_pos220

x	      &@

y	        

z	      @ó


drop_groupÂ6

element"Fire
	
id‡>

level!

name"Monster 8032
?
	spawn_pos220

x	      (@

y	        

z	      @ñ


drop_groupÊ6

element"Ice
	
id·>

level"

name"Monster 8033
?
	spawn_pos220

x	      *@

y	        

z	       @ú


drop_groupÁ6

element"	Lightning
	
id‚>

level#

name"Monster 8034
?
	spawn_pos220

x	      ,@

y	        

z	      "@õ


drop_groupË6

element
"Physical
	
id„>

level$

name"Monster 8035
?
	spawn_pos220

x	      .@

y	        

z	      $@ó


drop_groupÈ6

element"Fire
	
id‰>

level%

name"Monster 8036
?
	spawn_pos220

x	      0@

y	        

z	      &@ñ


drop_groupÍ6

element"Ice
	
idÂ>

level&

name"Monster 8037
?
	spawn_pos220

x	      1@

y	        

z	      (@ú


drop_groupÎ6

element"	Lightning
	
idÊ>

level'

name"Monster 8038
?
	spawn_pos220

x	      2@

y	        

z	      *@õ


drop_groupÏ6

element
"Physical
	
idÁ>

level(

name"Monster 8039
?
	spawn_pos220

x	      3@

y	        

z	      ,@ó


drop_groupŸ6

element"Fire
	
idË>

level)

name"Monster 8040
?
	spawn_pos220

x	        

y	        

z	        ñ


drop_group⁄6

element"Ice
	
idÈ>

level*

name"Monster 8041
?
	spawn_pos220

x	      ?

y	        

z	      ?ú


drop_group€6

element"	Lightning
	
idÍ>

level+

name"Monster 8042
?
	spawn_pos220

x	       @

y	        

z	       @õ


drop_group‹6

element
"Physical
	
idÎ>

level,

name"Monster 8043
?
	spawn_pos220

x	      @

y	        

z	      @ó


drop_group›6

element"Fire
	
idÏ>

level-

name"Monster 8044
?
	spawn_pos220

x	      @

y	        

z	      @ñ


drop_groupﬁ6

element"Ice
	
idÌ>

level.

name"Monster 8045
?
	spawn_pos220

x	      @

y	        

z	      @ú


drop_groupﬂ6

element"	Lightning
	
idÓ>

level/

name"Monster 8046
?
	spawn_pos220

x	      @

y	        

z	      @õ


drop_group‡6

element
"Physical
	
idÔ>

level0

name"Monster 8047
?
	spawn_pos220

x	      @

y	        

z	      @ó


drop_group·6

element"Fire
	
id>

level1

name"Monster 8048
?
	spawn_pos220

x	       @

y	        

z	       @ñ


drop_group‚6

element"Ice
	
idÒ>

level2

name"Monster 8049
?
	spawn_pos220

x	      "@

y	        

z	      "@ú


drop_group„6

element"	Lightning
	
idÚ>

level3

name"Monster 8050
?
	spawn_pos220

x	      $@

y	        

z	      $@õ


drop_group‰6

element
"Physical
	
idÛ>

level4

name"Monster 8051
?
	spawn_pos220

x	      &@

y	        

z	      &@ó


drop_groupÂ6

element"Fire
	
idÙ>

level5

name"Monster 8052
?
	spawn_pos220

x	      (@

y	        

z	      (@ñ


drop_groupÊ6

element"Ice
	
idı>

level6

name"Monster 8053
?
	spawn_pos220

x	      *@

y	        

z	      *@ú


drop_groupÁ6

element"	Lightning
	
idˆ>

level7

name"Monster 8054
?
	spawn_pos220

x	      ,@

y	        

z	      ,@õ


drop_groupË6

element
"Physical
	
id˜>

level8

name"Monster 8055
?
	spawn_pos220

x	      .@

y	        

z	        ó


drop_groupÈ6

element"Fire
	
id¯>

level9

name"Monster 8056
?
	spawn_pos220

x	      0@

y	        

z	      ?ñ


drop_groupÍ6

element"Ice
	
id˘>

level:

name"Monster 8057
?
	spawn_pos220

x	      1@

y	        

z	       @ú


drop_groupÎ6

element"	Lightning
	
id˙>

level;

name"Monster 8058
?
	spawn_pos220

x	      2@

y	        

z	      @õ


drop_groupÏ6

element
"Physical
	
id˚>

level<

name"Monster 8059
?
	spawn_pos220

x	      3@

y	        

z	      @ó


drop_groupŸ6

element"Fire
	
id¸>

level=

name"Monster 8060
?
	spawn_pos220

x	        

y	        

z	      @ñ


drop_group⁄6

element"Ice
	
id˝>

level>

name"Monster 8061
?
	spawn_pos220

x	      ?

y	        

z	      @ú


drop_group€6

element"	Lightning
	
id˛>

level?

name"Monster 8062
?
	spawn_pos220

x	       @

y	        

z	      @õ


drop_group‹6

element
"Physical
	
idˇ>

level@

name"Monster 8063
?
	spawn_pos220

x	      @

y	        

z	       @ó


drop_group›6

element"Fire
	
idÄ?

levelA

name"Monster 8064
?
	spawn_pos220

x	      @

y	        

z	      "@ñ


drop_groupﬁ6

element"Ice
	
idÅ?

levelB

name"Monster 8065
?
	spawn_pos220

x	      @

y	        

z	      $@ú


drop_groupﬂ6

element"	Lightning
	
idÇ?

levelC

name"Monster 8066
?
	spawn_pos220

x	      @

y	        

z	      &@õ


drop_group‡6

element
"Physical
	
idÉ?

levelD

name"Monster 8067
?
	spawn_pos220

x	      @

y	        

z	      (@ó


drop_group·6

element"Fire
	
idÑ?

levelE

name"Monster 8068
?
	spawn_pos220

x	       @

y	        

z	      *@ñ


drop_group‚6

element"Ice
	
idÖ?

levelF

name"Monster 8069
?
	spawn_pos220

x	      "@

y	        

z	      ,@ú


drop_group„6

element"	Lightning
	
idÜ?

levelG

name"Monster 8070
?
	spawn_pos220

x	      $@

y	        

z	        õ


drop_group‰6

element
"Physical
	
idá?

levelH

name"Monster 8071
?
	spawn_pos220

x	      &@

y	        

z	      ?ó


drop_groupÂ6

element"Fire
	
idà?

levelI

name"Monster 8072
?
	spawn_pos220

x	      (@

y	        

z	       @ñ


drop_groupÊ6

element"Ice
	
idâ?

levelJ

name"Monster 8073
?
	spawn_pos220

x	      *@

y	        

z	      @ú


drop_groupÁ6

element"	Lightning
	
idä?

levelK

name"Monster 8074
?
	spawn_pos220

x	      ,@

y	        

z	      @õ


drop_groupË6

element
"Physical
	
idã?

levelL

name"Monster 8075
?
	spawn_pos220

x	      .@

y	        

z	      @ó


drop_groupÈ6

element"Fire
	
idå?

levelM

name"Monster 8076
?
	spawn_pos220

x	      0@

y	        

z	      @ñ


drop_groupÍ6

element"Ice
	
idç?

levelN

name"Monster 8077
?
	spawn_pos220

x	      1@

y	        

z	      @ú


drop_groupÎ6

element"	Lightning
	
idé?

levelO

name"Monster 8078
?
	spawn_pos220

x	      2@

y	        

z	       @õ


drop_groupÏ6

element
"Physical
	
idè?

levelP

name"Monster 8079
?
	spawn_pos220

x	      3@

y	        

z	      "@ó


drop_groupŸ6

element"Fire
	
idê?

level

name"Monster 8080
?
	spawn_pos220

x	        

y	        

z	      $@*∑;
Stageª
[
first_clear_rewardsD*B
2

count

item_idç
2

count

item_idé
	
id©F
 
monster_ids*
Í>
Î>
Ï>

name"
Stage 9001

recommended_powerÏÀª
[
first_clear_rewardsD*B
2

count

item_idé
2

count

item_idè
	
id™F
 
monster_ids*
Î>
Ï>
Ì>

name"
Stage 9002

recommended_power¯Àª
[
first_clear_rewardsD*B
2

count

item_idè
2

count

item_idê
	
id´F
 
monster_ids*
Ï>
Ì>
Ó>

name"
Stage 9003

recommended_powerÑÃª
[
first_clear_rewardsD*B
2

count

item_idê
2

count

item_idë
	
id¨F
 
monster_ids*
Ì>
Ó>
Ô>

name"
Stage 9004

recommended_powerêÃª
[
first_clear_rewardsD*B
2

count

item_idë
2

count

item_idí
	
id≠F
 
monster_ids*
Ó>
Ô>
>

name"
Stage 9005

recommended_powerúÃª
[
first_clear_rewardsD*B
2

count

item_idí
2

count

item_idì
	
idÆF
 
monster_ids*
Ô>
>
Ò>

name"
Stage 9006

recommended_power®Ãª
[
first_clear_rewardsD*B
2

count

item_idì
2

count

item_idî
	
idØF
 
monster_ids*
>
Ò>
Ú>

name"
Stage 9007

recommended_power¥Ãª
[
first_clear_rewardsD*B
2

count

item_idî
2

count

item_idï
	
id∞F
 
monster_ids*
Ò>
Ú>
Û>

name"
Stage 9008

recommended_power¿Ãª
[
first_clear_rewardsD*B
2

count

item_idï
2

count

item_idñ
	
id±F
 
monster_ids*
Ú>
Û>
Ù>

name"
Stage 9009

recommended_powerÃÃª
[
first_clear_rewardsD*B
2

count

item_idñ
2

count

item_idó
	
id≤F
 
monster_ids*
Û>
Ù>
ı>

name"
Stage 9010

recommended_powerÿÃª
[
first_clear_rewardsD*B
2

count

item_idó
2

count

item_idò
	
id≥F
 
monster_ids*
Ù>
ı>
ˆ>

name"
Stage 9011

recommended_power‰Ãª
[
first_clear_rewardsD*B
2

count

item_idò
2

count

item_idô
	
id¥F
 
monster_ids*
ı>
ˆ>
˜>

name"
Stage 9012

recommended_powerÃª
[
first_clear_rewardsD*B
2

count

item_idô
2

count

item_idö
	
idµF
 
monster_ids*
ˆ>
˜>
¯>

name"
Stage 9013

recommended_power¸Ãª
[
first_clear_rewardsD*B
2

count

item_idö
2

count

item_idõ
	
id∂F
 
monster_ids*
˜>
¯>
˘>

name"
Stage 9014

recommended_poweràÕª
[
first_clear_rewardsD*B
2

count

item_idõ
2

count

item_idú
	
id∑F
 
monster_ids*
¯>
˘>
˙>

name"
Stage 9015

recommended_powerîÕª
[
first_clear_rewardsD*B
2

count

item_idú
2

count

item_idù
	
id∏F
 
monster_ids*
˘>
˙>
˚>

name"
Stage 9016

recommended_power†Õª
[
first_clear_rewardsD*B
2

count

item_idù
2

count

item_idû
	
idπF
 
monster_ids*
˙>
˚>
¸>

name"
Stage 9017

recommended_power¨Õª
[
first_clear_rewardsD*B
2

count

item_idû
2

count

item_idü
	
id∫F
 
monster_ids*
˚>
¸>
˝>

name"
Stage 9018

recommended_power∏Õª
[
first_clear_rewardsD*B
2

count

item_idü
2

count

item_id†
	
idªF
 
monster_ids*
¸>
˝>
˛>

name"
Stage 9019

recommended_powerƒÕª
[
first_clear_rewardsD*B
2

count

item_id†
2

count

item_id°
	
idºF
 
monster_ids*
˝>
˛>
ˇ>

name"
Stage 9020

recommended_power–Õª
[
first_clear_rewardsD*B
2

count

item_id°
2

count

item_id¢
	
idΩF
 
monster_ids*
˛>
ˇ>
Ä?

name"
Stage 9021

recommended_power‹Õª
[
first_clear_rewardsD*B
2

count

item_id¢
2

count

item_id£
	
idæF
 
monster_ids*
ˇ>
Ä?
Å?

name"
Stage 9022

recommended_powerËÕª
[
first_clear_rewardsD*B
2

count

item_id£
2

count

item_id§
	
idøF
 
monster_ids*
Ä?
Å?
Ç?

name"
Stage 9023

recommended_powerÙÕª
[
first_clear_rewardsD*B
2

count

item_id§
2

count

item_id•
	
id¿F
 
monster_ids*
Å?
Ç?
É?

name"
Stage 9024

recommended_powerÄŒª
[
first_clear_rewardsD*B
2

count

item_id•
2

count

item_id¶
	
id¡F
 
monster_ids*
Ç?
É?
Ñ?

name"
Stage 9025

recommended_poweråŒª
[
first_clear_rewardsD*B
2

count

item_id¶
2

count

item_idß
	
id¬F
 
monster_ids*
É?
Ñ?
Ö?

name"
Stage 9026

recommended_poweròŒª
[
first_clear_rewardsD*B
2

count

item_idß
2

count

item_id®
	
id√F
 
monster_ids*
Ñ?
Ö?
Ü?

name"
Stage 9027

recommended_power§Œª
[
first_clear_rewardsD*B
2

count

item_id®
2

count

item_id©
	
idƒF
 
monster_ids*
Ö?
Ü?
á?

name"
Stage 9028

recommended_power∞Œª
[
first_clear_rewardsD*B
2

count

item_id©
2

count

item_id™
	
id≈F
 
monster_ids*
Ü?
á?
à?

name"
Stage 9029

recommended_powerºŒª
[
first_clear_rewardsD*B
2

count

item_id™
2

count

item_id´
	
id∆F
 
monster_ids*
á?
à?
â?

name"
Stage 9030

recommended_power»Œª
[
first_clear_rewardsD*B
2

count

item_id´
2

count

item_id¨
	
id«F
 
monster_ids*
à?
â?
ä?

name"
Stage 9031

recommended_power‘Œª
[
first_clear_rewardsD*B
2

count

item_id¨
2

count

item_id≠
	
id»F
 
monster_ids*
â?
ä?
ã?

name"
Stage 9032

recommended_power‡Œª
[
first_clear_rewardsD*B
2

count

item_id≠
2

count

item_idÆ
	
id…F
 
monster_ids*
ä?
ã?
å?

name"
Stage 9033

recommended_powerÏŒª
[
first_clear_rewardsD*B
2

count

item_idÆ
2

count

item_idØ
	
id F
 
monster_ids*
ã?
å?
ç?

name"
Stage 9034

recommended_power¯Œª
[
first_clear_rewardsD*B
2

count

item_idØ
2

count

item_id∞
	
idÀF
 
monster_ids*
å?
ç?
é?

name"
Stage 9035

recommended_powerÑœª
[
first_clear_rewardsD*B
2

count

item_id∞
2

count

item_id±
	
idÃF
 
monster_ids*
ç?
é?
è?

name"
Stage 9036

recommended_powerêœª
[
first_clear_rewardsD*B
2

count

item_id±
2

count

item_id≤
	
idÕF
 
monster_ids*
é?
è?
ê?

name"
Stage 9037

recommended_powerúœª
[
first_clear_rewardsD*B
2

count

item_id≤
2

count

item_id≥
	
idŒF
 
monster_ids*
è?
ê?
¡>

name"
Stage 9038

recommended_power®œª
[
first_clear_rewardsD*B
2

count

item_id≥
2

count

item_id¥
	
idœF
 
monster_ids*
ê?
¡>
¬>

name"
Stage 9039

recommended_power¥œª
[
first_clear_rewardsD*B
2

count

item_id¥
2

count

item_idµ
	
id–F
 
monster_ids*
¡>
¬>
√>

name"
Stage 9040

recommended_power¿œ*˝$
StageReward9

count

item_idç
	
seq

stage_id©F9

count

item_idé
	
seq

stage_id©F9

count

item_idé
	
seq

stage_id™F9

count

item_idè
	
seq

stage_id™F9

count

item_idè
	
seq

stage_id´F9

count

item_idê
	
seq

stage_id´F9

count

item_idê
	
seq

stage_id¨F9

count

item_idë
	
seq

stage_id¨F9

count

item_idë
	
seq

stage_id≠F9

count

item_idí
	
seq

stage_id≠F9

count

item_idí
	
seq

stage_idÆF9

count

item_idì
	
seq

stage_idÆF9

count

item_idì
	
seq

stage_idØF9

count

item_idî
	
seq

stage_idØF9

count

item_idî
	
seq

stage_id∞F9

count

item_idï
	
seq

stage_id∞F9

count

item_idï
	
seq

stage_id±F9

count

item_idñ
	
seq

stage_id±F9

count

item_idñ
	
seq

stage_id≤F9

count

item_idó
	
seq

stage_id≤F9

count

item_idó
	
seq

stage_id≥F9

count

item_idò
	
seq

stage_id≥F9

count

item_idò
	
seq

stage_id¥F9

count

item_idô
	
seq

stage_id¥F9

count

item_idô
	
seq

stage_idµF9

count

item_idö
	
seq

stage_idµF9

count

item_idö
	
seq

stage_id∂F9

count

item_idõ
	
seq

stage_id∂F9

count

item_idõ
	
seq

stage_id∑F9

count

item_idú
	
seq

stage_id∑F9

count

item_idú
	
seq

stage_id∏F9

count

item_idù
	
seq

stage_id∏F9

count

item_idù
	
seq

stage_idπF9

count

item_idû
	
seq

stage_idπF9

count

item_idû
	
seq

stage_id∫F9

count

item_idü
	
seq

stage_id∫F9

count

item_idü
	
seq

stage_idªF9

count

item_id†
	
seq

stage_idªF9

count

item_id†
	
seq

stage_idºF9

count

item_id°
	
seq

stage_idºF9

count

item_id°
	
seq

stage_idΩF9

count

item_id¢
	
seq

stage_idΩF9

count

item_id¢
	
seq

stage_idæF9

count

item_id£
	
seq

stage_idæF9

count

item_id£
	
seq

stage_idøF9

count

item_id§
	
seq

stage_idøF9

count

item_id§
	
seq

stage_id¿F9

count

item_id•
	
seq

stage_id¿F9

count

item_id•
	
seq

stage_id¡F9

count

item_id¶
	
seq

stage_id¡F9

count

item_id¶
	
seq

stage_id¬F9

count

item_idß
	
seq

stage_id¬F9

count

item_idß
	
seq

stage_id√F9

count

item_id®
	
seq

stage_id√F9

count

item_id®
	
seq

stage_idƒF9

count

item_id©
	
seq

stage_idƒF9

count

item_id©
	
seq

stage_id≈F9

count

item_id™
	
seq

stage_id≈F9

count

item_id™
	
seq

stage_id∆F9

count

item_id´
	
seq

stage_id∆F9

count

item_id´
	
seq

stage_id«F9

count

item_id¨
	
seq

stage_id«F9

count

item_id¨
	
seq

stage_id»F9

count

item_id≠
	
seq

stage_id»F9

count

item_id≠
	
seq

stage_id…F9

count

item_idÆ
	
seq

stage_id…F9

count

item_idÆ
	
seq

stage_id F9

count

item_idØ
	
seq

stage_id F9

count

item_idØ
	
seq

stage_idÀF9

count

item_id∞
	
seq

stage_idÀF9

count

item_id∞
	
seq

stage_idÃF9

count

item_id±
	
seq

stage_idÃF9

count

item_id±
	
seq

stage_idÕF9

count

item_id≤
	
seq

stage_idÕF9

count

item_id≤
	
seq

stage_idŒF9

count

item_id≥
	
seq

stage_idŒF9

count

item_id≥
	
seq

stage_idœF9

count

item_id¥
	
seq

stage_idœF9

count

item_id¥
	
seq

stage_id–F9

count

item_idµ
	
seq

stage_id–F*î

Dungeon~
7

entry_cost)2'

countd

id 

kind"Gold
	
idùJ

name"	Dungeon 1
#
	stage_ids*
©F
™F
´F
¨F
8

entry_cost*2(

count»

id 

kind"Gold
	
idûJ

name"	Dungeon 2
#
	stage_ids*
≠F
ÆF
ØF
∞F
8

entry_cost*2(

count¨

id 

kind"Gold
	
idüJ

name"	Dungeon 3
#
	stage_ids*
±F
≤F
≥F
¥F
8

entry_cost*2(

countê

id 

kind"Gold
	
id†J

name"	Dungeon 4
#
	stage_ids*
µF
∂F
∑F
∏F
8

entry_cost*2(

countÙ

id 

kind"Gold
	
id°J

name"	Dungeon 5
#
	stage_ids*
πF
∫F
ªF
ºF
8

entry_cost*2(

countÿ

id 

kind"Gold
	
id¢J

name"	Dungeon 6
#
	stage_ids*
ΩF
æF
øF
¿F
8

entry_cost*2(

countº

id 

kind"Gold
	
id£J

name"	Dungeon 7
#
	stage_ids*
¡F
¬F
√F
ƒF
8

entry_cost*2(

count†

id 

kind"Gold
	
id§J

name"	Dungeon 8
#
	stage_ids*
≈F
∆F
«F
»F
8

entry_cost*2(

countÑ

id 

kind"Gold
	
id•J

name"	Dungeon 9
#
	stage_ids*
…F
 F
ÀF
ÃFÄ
8

entry_cost*2(

countË

id 

kind"Gold
	
id¶J

name"
Dungeon 10
#
	stage_ids*
ÕF
ŒF
œF
–F*ˆ1
Quest˛
	
idâ'


quest_type"Main

required_itemÈ
O
rewardsD*B
2

count

item_idπ
2

count

item_id—
?
	start_pos220

x	      (@

y	        

z	      @

title"First Trial

unlock_skills
*
e
f›
	
idä'


quest_type"Daily

required_itemÍ
.
rewards#*!
2

count

item_idÍ
?
	start_pos220

x	       @

y	        

z	       @

title"Crystal Supply

unlock_skills*
f˛
	
idã'


quest_type"Daily

required_itemö
O
rewardsD*B
2

count

item_idõ
2

count

item_idú
?
	start_pos220

x	      @

y	        

z	      7@

title"
Quest 5003

unlock_skills
*
|
}˝
	
idå'


quest_type"Main

required_itemõ
O
rewardsD*B
2

count

item_idú
2

count

item_idù
?
	start_pos220

x	      @

y	        

z	      8@

title"
Quest 5004

unlock_skills
*
}
~˝
	
idç'


quest_type"Side

required_itemú
O
rewardsD*B
2

count

item_idù
2

count

item_idû
?
	start_pos220

x	      @

y	        

z	      9@

title"
Quest 5005

unlock_skills
*
~
ˇ
	
idé'


quest_type"Daily

required_itemù
O
rewardsD*B
2

count

item_idû
2

count

item_idü
?
	start_pos220

x	      @

y	        

z	      :@

title"
Quest 5006

unlock_skills*	

Äˇ
	
idè'


quest_type"Main

required_itemû
O
rewardsD*B
2

count

item_idü
2

count

item_id†
?
	start_pos220

x	      @

y	        

z	      ;@

title"
Quest 5007

unlock_skills*

Ä
Åˇ
	
idê'


quest_type"Side

required_itemü
O
rewardsD*B
2

count

item_id†
2

count

item_id°
?
	start_pos220

x	       @

y	        

z	      <@

title"
Quest 5008

unlock_skills*

Å
Çˇ
	
idë'


quest_type"Daily

required_item†
O
rewardsD*B
2

count

item_id°
2

count

item_id¢
?
	start_pos220

x	      "@

y	        

z	      =@

title"
Quest 5009

unlock_skills*	
Ç
e˝
	
idí'


quest_type"Main

required_item°
O
rewardsD*B
2

count

item_id¢
2

count

item_id£
?
	start_pos220

x	      $@

y	        

z	        

title"
Quest 5010

unlock_skills
*
e
f˝
	
idì'


quest_type"Side

required_item¢
O
rewardsD*B
2

count

item_id£
2

count

item_id§
?
	start_pos220

x	      &@

y	        

z	      ?

title"
Quest 5011

unlock_skills
*
f
g˛
	
idî'


quest_type"Daily

required_item£
O
rewardsD*B
2

count

item_id§
2

count

item_id•
?
	start_pos220

x	      (@

y	        

z	       @

title"
Quest 5012

unlock_skills
*
g
h˝
	
idï'


quest_type"Main

required_item§
O
rewardsD*B
2

count

item_id•
2

count

item_id¶
?
	start_pos220

x	      *@

y	        

z	      @

title"
Quest 5013

unlock_skills
*
h
i˝
	
idñ'


quest_type"Side

required_item•
O
rewardsD*B
2

count

item_id¶
2

count

item_idß
?
	start_pos220

x	      ,@

y	        

z	      @

title"
Quest 5014

unlock_skills
*
i
j˛
	
idó'


quest_type"Daily

required_item¶
O
rewardsD*B
2

count

item_idß
2

count

item_id®
?
	start_pos220

x	      .@

y	        

z	      @

title"
Quest 5015

unlock_skills
*
j
k˝
	
idò'


quest_type"Main

required_itemß
O
rewardsD*B
2

count

item_id®
2

count

item_id©
?
	start_pos220

x	      0@

y	        

z	      @

title"
Quest 5016

unlock_skills
*
k
l˝
	
idô'


quest_type"Side

required_item®
O
rewardsD*B
2

count

item_id©
2

count

item_id™
?
	start_pos220

x	      1@

y	        

z	      @

title"
Quest 5017

unlock_skills
*
l
m˛
	
idö'


quest_type"Daily

required_item©
O
rewardsD*B
2

count

item_id™
2

count

item_id´
?
	start_pos220

x	      2@

y	        

z	       @

title"
Quest 5018

unlock_skills
*
m
n˝
	
idõ'


quest_type"Main

required_item™
O
rewardsD*B
2

count

item_id´
2

count

item_id¨
?
	start_pos220

x	      3@

y	        

z	      "@

title"
Quest 5019

unlock_skills
*
n
o˝
	
idú'


quest_type"Side

required_item´
O
rewardsD*B
2

count

item_id¨
2

count

item_id≠
?
	start_pos220

x	      4@

y	        

z	      $@

title"
Quest 5020

unlock_skills
*
o
p˛
	
idù'


quest_type"Daily

required_item¨
O
rewardsD*B
2

count

item_id≠
2

count

item_idÆ
?
	start_pos220

x	      5@

y	        

z	      &@

title"
Quest 5021

unlock_skills
*
p
q˝
	
idû'


quest_type"Main

required_item≠
O
rewardsD*B
2

count

item_idÆ
2

count

item_idØ
?
	start_pos220

x	      6@

y	        

z	      (@

title"
Quest 5022

unlock_skills
*
q
r˝
	
idü'


quest_type"Side

required_itemÆ
O
rewardsD*B
2

count

item_idØ
2

count

item_id∞
?
	start_pos220

x	      7@

y	        

z	      *@

title"
Quest 5023

unlock_skills
*
r
s˛
	
id†'


quest_type"Daily

required_itemØ
O
rewardsD*B
2

count

item_id∞
2

count

item_id±
?
	start_pos220

x	      8@

y	        

z	      ,@

title"
Quest 5024

unlock_skills
*
s
t˝
	
id°'


quest_type"Main

required_item∞
O
rewardsD*B
2

count

item_id±
2

count

item_id≤
?
	start_pos220

x	      9@

y	        

z	      .@

title"
Quest 5025

unlock_skills
*
t
u*ÿ
QuestReward9

count

item_idπ

quest_idâ'
	
seq9

count

item_id—

quest_idâ'
	
seq9

count

item_idÍ

quest_idä'
	
seq9

count

item_idõ

quest_idã'
	
seq9

count

item_idú

quest_idã'
	
seq9

count

item_idú

quest_idå'
	
seq9

count

item_idù

quest_idå'
	
seq9

count

item_idù

quest_idç'
	
seq9

count

item_idû

quest_idç'
	
seq9

count

item_idû

quest_idé'
	
seq9

count

item_idü

quest_idé'
	
seq9

count

item_idü

quest_idè'
	
seq9

count

item_id†

quest_idè'
	
seq9

count

item_id†

quest_idê'
	
seq9

count

item_id°

quest_idê'
	
seq9

count

item_id°

quest_idë'
	
seq9

count

item_id¢

quest_idë'
	
seq9

count

item_id¢

quest_idí'
	
seq9

count

item_id£

quest_idí'
	
seq9

count

item_id£

quest_idì'
	
seq9

count

item_id§

quest_idì'
	
seq9

count

item_id§

quest_idî'
	
seq9

count

item_id•

quest_idî'
	
seq9

count

item_id•

quest_idï'
	
seq9

count

item_id¶

quest_idï'
	
seq9

count

item_id¶

quest_idñ'
	
seq9

count

item_idß

quest_idñ'
	
seq9

count

item_idß

quest_idó'
	
seq9

count

item_id®

quest_idó'
	
seq9

count

item_id®

quest_idò'
	
seq9

count

item_id©

quest_idò'
	
seq9

count

item_id©

quest_idô'
	
seq9

count

item_id™

quest_idô'
	
seq9

count

item_id™

quest_idö'
	
seq9

count

item_id´

quest_idö'
	
seq9

count

item_id´

quest_idõ'
	
seq9

count

item_id¨

quest_idõ'
	
seq9

count

item_id¨

quest_idú'
	
seq9

count

item_id≠

quest_idú'
	
seq9

count

item_id≠

quest_idù'
	
seq9

count

item_idÆ

quest_idù'
	
seq9

count

item_idÆ

quest_idû'
	
seq9

count

item_idØ

quest_idû'
	
seq9

count

item_idØ

quest_idü'
	
seq9

count

item_id∞

quest_idü'
	
seq9

count

item_id∞

quest_id†'
	
seq9

count

item_id±

quest_id†'
	
seq9

count

item_id±

quest_id°'
	
seq9

count

item_id≤

quest_id°'
	
seq*Í'
LevelExp.
	
expd

level

unlock_feature8/


expê

level

unlock_feature8/


expÑ

level

unlock_feature8/


exp¿

level

unlock_feature8/


expƒ

level

unlock_feature8/


expê

level

unlock_feature8/


exp§&

level

unlock_feature8/


expÄ2

level

unlock_feature8/


exp§?

level	

unlock_feature89


expêN

level


unlock_feature"
feature_10/


expƒ^

level

unlock_feature8/


exp¿p

level

unlock_feature80

expÑÑ

level

unlock_feature80

expêô

level

unlock_feature80

exp‰Ø

level

unlock_feature80

expÄ»

level

unlock_feature80

exp‰·

level

unlock_feature80

expê˝

level

unlock_feature80

expÑö

level

unlock_feature8:

exp¿∏

level

unlock_feature"
feature_200

expƒÿ

level

unlock_feature80

expê˙

level

unlock_feature80

exp§ù

level

unlock_feature80

expÄ¬

level

unlock_feature80

exp§Ë

level

unlock_feature80

expêê

level

unlock_feature80

expƒπ

level

unlock_feature80

exp¿‰

level

unlock_feature80

expÑë

level

unlock_feature8:

expêø

level

unlock_feature"
feature_300

exp‰Ó

level

unlock_feature80

expÄ†

level 

unlock_feature80

exp‰“

level!

unlock_feature80

expêá

level"

unlock_feature80

expÑΩ

level#

unlock_feature80

exp¿Ù

level$

unlock_feature80

expƒ≠

level%

unlock_feature80

expêË

level&

unlock_feature80

exp§§	

level'

unlock_feature8:

expÄ‚	

level(

unlock_feature"
feature_400

exp§°


level)

unlock_feature80

expê‚


level*

unlock_feature80

expƒ§

level+

unlock_feature80

exp¿Ë

level,

unlock_feature80

expÑÆ

level-

unlock_feature80

expêı

level.

unlock_feature80

exp‰Ω

level/

unlock_feature80

expÄà

level0

unlock_feature80

exp‰”

level1

unlock_feature8:

expê°

level2

unlock_feature"
feature_500

expÑ

level3

unlock_feature80

exp¿¿

level4

unlock_feature80

expƒí

level5

unlock_feature80

expêÊ

level6

unlock_feature80

exp§ª

level7

unlock_feature80

expÄí

level8

unlock_feature80

exp§Í

level9

unlock_feature80

expêƒ

level:

unlock_feature80

expƒü

level;

unlock_feature8:

exp¿¸

level<

unlock_feature"
feature_600

expÑ€

level=

unlock_feature80

expêª

level>

unlock_feature80

exp‰ú

level?

unlock_feature80

expÄÄ

level@

unlock_feature80

exp‰‰

levelA

unlock_feature80

expêÀ

levelB

unlock_feature80

expÑ≥

levelC

unlock_feature80

exp¿ú

levelD

unlock_feature80

expƒá

levelE

unlock_feature8:

expêÙ

levelF

unlock_feature"
feature_700

exp§‚

levelG

unlock_feature80

expÄ“

levelH

unlock_feature80

exp§√ 

levelI

unlock_feature80

expê∂!

levelJ

unlock_feature80

expƒ™"

levelK

unlock_feature80

exp¿†#

levelL

unlock_feature80

expÑò$

levelM

unlock_feature80

expêë%

levelN

unlock_feature80

exp‰ã&

levelO

unlock_feature8:

expÄà'

levelP

unlock_feature"
feature_800

exp‰Ö(

levelQ

unlock_feature80

expêÖ)

levelR

unlock_feature80

expÑÜ*

levelS

unlock_feature80

exp¿à+

levelT

unlock_feature80

expƒå,

levelU

unlock_feature80

expêí-

levelV

unlock_feature80

exp§ô.

levelW

unlock_feature80

expÄ¢/

levelX

unlock_feature80

exp§¨0

levelY

unlock_feature8:

expê∏1

levelZ

unlock_feature"
feature_900

expƒ≈2

level[

unlock_feature80

exp¿‘3

level\

unlock_feature80

expÑÂ4

level]

unlock_feature80

expê˜5

level^

unlock_feature80

exp‰ä7

level_

unlock_feature80

expÄ†8

level`

unlock_feature80

exp‰∂9

levela

unlock_feature80

expêœ:

levelb

unlock_feature80

expÑÈ;

levelc

unlock_feature8;

exp¿Ñ=

leveld

unlock_feature"feature_100*π
Achievementm
	
id±m
3
reward)2'

count2

id 

kind"Gold

target_countË

	title_key	"loc_001m
	
id≤m
3
reward)2'

countd

id 

kind"Gold

target_count–

	title_key	"loc_002n
	
id≥m
4
reward*2(

countñ

id 

kind"Gold

target_count∏

	title_key	"loc_003n
	
id¥m
4
reward*2(

count»

id 

kind"Gold

target_count†

	title_key	"loc_004n
	
idµm
4
reward*2(

count˙

id 

kind"Gold

target_countà'

	title_key	"loc_005n
	
id∂m
4
reward*2(

count¨

id 

kind"Gold

target_count.

	title_key	"loc_006n
	
id∑m
4
reward*2(

countﬁ

id 

kind"Gold

target_countÿ6

	title_key	"loc_007n
	
id∏m
4
reward*2(

countê

id 

kind"Gold

target_count¿>

	title_key	"loc_008n
	
idπm
4
reward*2(

count¬

id 

kind"Gold

target_count®F

	title_key	"loc_009n
	
id∫m
4
reward*2(

countÙ

id 

kind"Gold

target_countêN

	title_key	"loc_010n
	
idªm
4
reward*2(

count¶

id 

kind"Gold

target_count¯U

	title_key	"loc_011n
	
idºm
4
reward*2(

countÿ

id 

kind"Gold

target_count‡]

	title_key	"loc_012n
	
idΩm
4
reward*2(

countä

id 

kind"Gold

target_count»e

	title_key	"loc_013n
	
idæm
4
reward*2(

countº

id 

kind"Gold

target_count∞m

	title_key	"loc_014n
	
idøm
4
reward*2(

countÓ

id 

kind"Gold

target_countòu

	title_key	"loc_015n
	
id¿m
4
reward*2(

count†

id 

kind"Gold

target_countÄ}

	title_key	"loc_016o
	
id¡m
4
reward*2(

count“

id 

kind"Gold

target_countËÑ

	title_key	"loc_017o
	
id¬m
4
reward*2(

countÑ

id 

kind"Gold

target_count–å

	title_key	"loc_018o
	
id√m
4
reward*2(

count∂

id 

kind"Gold

target_count∏î

	title_key	"loc_019o
	
idƒm
4
reward*2(

countË

id 

kind"Gold

target_count†ú

	title_key	"loc_020o
	
id≈m
4
reward*2(

countö

id 

kind"Gold

target_countà§

	title_key	"loc_021o
	
id∆m
4
reward*2(

countÃ

id 

kind"Gold

target_count´

	title_key	"loc_022o
	
id«m
4
reward*2(

count˛

id 

kind"Gold

target_countÿ≥

	title_key	"loc_023o
	
id»m
4
reward*2(

count∞	

id 

kind"Gold

target_count¿ª

	title_key	"loc_024o
	
id…m
4
reward*2(

count‚	

id 

kind"Gold

target_count®√

	title_key	"loc_025o
	
id m
4
reward*2(

countî


id 

kind"Gold

target_countêÀ

	title_key	"loc_026o
	
idÀm
4
reward*2(

count∆


id 

kind"Gold

target_count¯“

	title_key	"loc_027o
	
idÃm
4
reward*2(

count¯


id 

kind"Gold

target_count‡⁄

	title_key	"loc_028o
	
idÕm
4
reward*2(

count™

id 

kind"Gold

target_count»‚

	title_key	"loc_029o
	
idŒm
4
reward*2(

count‹

id 

kind"Gold

target_count∞Í

	title_key	"loc_030*ú	
VipLevelr
4
cost,2*

countd

id 

kind	"Diamond

level
-
perks$*"
"fast_battle
"shop_discount_1s
5
cost-2+

count»

id 

kind	"Diamond

level
-
perks$*"
"fast_battle
"shop_discount_2s
5
cost-2+

count¨

id 

kind	"Diamond

level
-
perks$*"
"fast_battle
"shop_discount_3s
5
cost-2+

countê

id 

kind	"Diamond

level
-
perks$*"
"fast_battle
"shop_discount_4s
5
cost-2+

countÙ

id 

kind	"Diamond

level
-
perks$*"
"fast_battle
"shop_discount_5s
5
cost-2+

countÿ

id 

kind	"Diamond

level
-
perks$*"
"fast_battle
"shop_discount_6s
5
cost-2+

countº

id 

kind	"Diamond

level
-
perks$*"
"fast_battle
"shop_discount_7s
5
cost-2+

count†

id 

kind	"Diamond

level
-
perks$*"
"fast_battle
"shop_discount_8s
5
cost-2+

countÑ

id 

kind	"Diamond

level	
-
perks$*"
"fast_battle
"shop_discount_9t
5
cost-2+

countË

id 

kind	"Diamond

level

.
perks%*#
"fast_battle
"shop_discount_10*⁄
GameSettings…
&
daily_bonus_items*
È
Í
—

daily_reset_hour

gravity	£í:ù#@
j
maintenance[2Y

duration_minutesZ

reason"Season rollout
#
	starts_at"2026-06-01T03:00:00Z
z
spawn_pointsj*h
220

x	        

y	        

z	        
220

x	      (@

y	        

z	       @
?
	spawn_pos220

x	        

y	        

z	        

starter_items*

È
—

starting_goldd

version	"2026.05*Ñ
MaintenanceWindowo

duration_minutesZ

reason"Season rollout
#
	starts_at"2026-06-01T03:00:00Z

version	"2026.05*ï
MailTemplate°

body_key	"loc_021
	
idôu

	mail_type"Event
O
rewardsD*B
2

count

item_idÌ
2

count

item_idÓ

	title_key	"loc_001®

body_key	"loc_022
	
idöu

	mail_type"Compensation
O
rewardsD*B
2

count

item_idÓ
2

count

item_idÔ

	title_key	"loc_002¢

body_key	"loc_023
	
idõu

	mail_type"System
O
rewardsD*B
2

count

item_idÔ
2

count

item_id

	title_key	"loc_003°

body_key	"loc_024
	
idúu

	mail_type"Event
O
rewardsD*B
2

count

item_id
2

count

item_idÒ

	title_key	"loc_004®

body_key	"loc_025
	
idùu

	mail_type"Compensation
O
rewardsD*B
2

count

item_idÒ
2

count

item_idÚ

	title_key	"loc_005¢

body_key	"loc_026
	
idûu

	mail_type"System
O
rewardsD*B
2

count

item_idÚ
2

count

item_idÛ

	title_key	"loc_006°

body_key	"loc_027
	
idüu

	mail_type"Event
O
rewardsD*B
2

count

item_idÛ
2

count

item_idÙ

	title_key	"loc_007®

body_key	"loc_028
	
id†u

	mail_type"Compensation
O
rewardsD*B
2

count

item_idÙ
2

count

item_idı

	title_key	"loc_008¢

body_key	"loc_029
	
id°u

	mail_type"System
O
rewardsD*B
2

count

item_idı
2

count

item_idˆ

	title_key	"loc_009°

body_key	"loc_030
	
id¢u

	mail_type"Event
O
rewardsD*B
2

count

item_idˆ
2

count

item_id˜

	title_key	"loc_010®

body_key	"loc_031
	
id£u

	mail_type"Compensation
O
rewardsD*B
2

count

item_id˜
2

count

item_id¯

	title_key	"loc_011¢

body_key	"loc_032
	
id§u

	mail_type"System
O
rewardsD*B
2

count

item_id¯
2

count

item_id˘

	title_key	"loc_012°

body_key	"loc_033
	
id•u

	mail_type"Event
O
rewardsD*B
2

count

item_id˘
2

count

item_id˙

	title_key	"loc_013®

body_key	"loc_034
	
id¶u

	mail_type"Compensation
O
rewardsD*B
2

count

item_id˙
2

count

item_id˚

	title_key	"loc_014¢

body_key	"loc_035
	
idßu

	mail_type"System
O
rewardsD*B
2

count

item_id˚
2

count

item_id¸

	title_key	"loc_015°

body_key	"loc_036
	
id®u

	mail_type"Event
O
rewardsD*B
2

count

item_id¸
2

count

item_id˝

	title_key	"loc_016®

body_key	"loc_037
	
id©u

	mail_type"Compensation
O
rewardsD*B
2

count

item_id˝
2

count

item_id˛

	title_key	"loc_017¢

body_key	"loc_038
	
id™u

	mail_type"System
O
rewardsD*B
2

count

item_id˛
2

count

item_idˇ

	title_key	"loc_018°

body_key	"loc_039
	
id´u

	mail_type"Event
O
rewardsD*B
2

count

item_idˇ
2

count

item_idÄ

	title_key	"loc_019®

body_key	"loc_040
	
id¨u

	mail_type"Compensation
O
rewardsD*B
2

count

item_idÄ
2

count

item_idÅ

	title_key	"loc_020*ú

MailReward8

count

item_idÌ

mail_idôu
	
seq8

count

item_idÓ

mail_idôu
	
seq8

count

item_idÓ

mail_idöu
	
seq8

count

item_idÔ

mail_idöu
	
seq8

count

item_idÔ

mail_idõu
	
seq8

count

item_id

mail_idõu
	
seq8

count

item_id

mail_idúu
	
seq8

count

item_idÒ

mail_idúu
	
seq8

count

item_idÒ

mail_idùu
	
seq8

count

item_idÚ

mail_idùu
	
seq8

count

item_idÚ

mail_idûu
	
seq8

count

item_idÛ

mail_idûu
	
seq8

count

item_idÛ

mail_idüu
	
seq8

count

item_idÙ

mail_idüu
	
seq8

count

item_idÙ

mail_id†u
	
seq8

count

item_idı

mail_id†u
	
seq8

count

item_idı

mail_id°u
	
seq8

count

item_idˆ

mail_id°u
	
seq8

count

item_idˆ

mail_id¢u
	
seq8

count

item_id˜

mail_id¢u
	
seq8

count

item_id˜

mail_id£u
	
seq8

count

item_id¯

mail_id£u
	
seq8

count

item_id¯

mail_id§u
	
seq8

count

item_id˘

mail_id§u
	
seq8

count

item_id˘

mail_id•u
	
seq8

count

item_id˙

mail_id•u
	
seq8

count

item_id˙

mail_id¶u
	
seq8

count

item_id˚

mail_id¶u
	
seq8

count

item_id˚

mail_idßu
	
seq8

count

item_id¸

mail_idßu
	
seq8

count

item_id¸

mail_id®u
	
seq8

count

item_id˝

mail_id®u
	
seq8

count

item_id˝

mail_id©u
	
seq8

count

item_id˛

mail_id©u
	
seq8

count

item_id˛

mail_id™u
	
seq8

count

item_idˇ

mail_id™u
	
seq8

count

item_idˇ

mail_id´u
	
seq8

count

item_idÄ

mail_id´u
	
seq8

count

item_idÄ

mail_id¨u
	
seq8

count

item_idÅ

mail_id¨u
	
seq*¯
Dialogue\
	
idÅ}
5
lines,**
"dialogue line 1-1
"dialogue line 1-2

speaker_key	"loc_041\
	
idÇ}
5
lines,**
"dialogue line 2-1
"dialogue line 2-2

speaker_key	"loc_042\
	
idÉ}
5
lines,**
"dialogue line 3-1
"dialogue line 3-2

speaker_key	"loc_043\
	
idÑ}
5
lines,**
"dialogue line 4-1
"dialogue line 4-2

speaker_key	"loc_044\
	
idÖ}
5
lines,**
"dialogue line 5-1
"dialogue line 5-2

speaker_key	"loc_045\
	
idÜ}
5
lines,**
"dialogue line 6-1
"dialogue line 6-2

speaker_key	"loc_046\
	
idá}
5
lines,**
"dialogue line 7-1
"dialogue line 7-2

speaker_key	"loc_047\
	
idà}
5
lines,**
"dialogue line 8-1
"dialogue line 8-2

speaker_key	"loc_048\
	
idâ}
5
lines,**
"dialogue line 9-1
"dialogue line 9-2

speaker_key	"loc_049^
	
idä}
7
lines.*,
"dialogue line 10-1
"dialogue line 10-2

speaker_key	"loc_050^
	
idã}
7
lines.*,
"dialogue line 11-1
"dialogue line 11-2

speaker_key	"loc_051^
	
idå}
7
lines.*,
"dialogue line 12-1
"dialogue line 12-2

speaker_key	"loc_052^
	
idç}
7
lines.*,
"dialogue line 13-1
"dialogue line 13-2

speaker_key	"loc_053^
	
idé}
7
lines.*,
"dialogue line 14-1
"dialogue line 14-2

speaker_key	"loc_054^
	
idè}
7
lines.*,
"dialogue line 15-1
"dialogue line 15-2

speaker_key	"loc_055^
	
idê}
7
lines.*,
"dialogue line 16-1
"dialogue line 16-2

speaker_key	"loc_056^
	
idë}
7
lines.*,
"dialogue line 17-1
"dialogue line 17-2

speaker_key	"loc_057^
	
idí}
7
lines.*,
"dialogue line 18-1
"dialogue line 18-2

speaker_key	"loc_058^
	
idì}
7
lines.*,
"dialogue line 19-1
"dialogue line 19-2

speaker_key	"loc_059^
	
idî}
7
lines.*,
"dialogue line 20-1
"dialogue line 20-2

speaker_key	"loc_060*Ö*
	EventRuleâ
¶
actionsö*ó
220

count

item_idÔ

type	"AddItem
523

buff_idÚ.

duration

type	"AddBuff
*2(

stage_id™F

type"UnlockStage
:
	condition-2+

quest_idä'

type"QuestCompleted


idÈÑ

name"Event Rule 1é
¶
actionsö*ó
220

count

item_id

type	"AddItem
523

buff_idÛ.

duration

type	"AddBuff
*2(

stage_id´F

type"UnlockStage
?
	condition220

count

item_idÌ

type	"HasItem


idÍÑ

name"Event Rule 2É
¶
actionsö*ó
220

count

item_idÒ

type	"AddItem
523

buff_idÙ.

duration

type	"AddBuff
*2(

stage_id¨F

type"UnlockStage
4
	condition'2%

level

type"LevelAtLeast


idÎÑ

name"Event Rule 3â
¶
actionsö*ó
220

count

item_idÚ

type	"AddItem
523

buff_idı.

duration	

type	"AddBuff
*2(

stage_id≠F

type"UnlockStage
:
	condition-2+

quest_idç'

type"QuestCompleted


idÏÑ

name"Event Rule 4é
¶
actionsö*ó
220

count

item_idÛ

type	"AddItem
523

buff_idˆ.

duration

type	"AddBuff
*2(

stage_idÆF

type"UnlockStage
?
	condition220

count

item_id

type	"HasItem


idÌÑ

name"Event Rule 5É
¶
actionsö*ó
220

count

item_idÙ

type	"AddItem
523

buff_id˜.

duration

type	"AddBuff
*2(

stage_idØF

type"UnlockStage
4
	condition'2%

level

type"LevelAtLeast


idÓÑ

name"Event Rule 6â
¶
actionsö*ó
220

count

item_idı

type	"AddItem
523

buff_id¯.

duration

type	"AddBuff
*2(

stage_id∞F

type"UnlockStage
:
	condition-2+

quest_idê'

type"QuestCompleted


idÔÑ

name"Event Rule 7é
¶
actionsö*ó
220

count

item_idˆ

type	"AddItem
523

buff_id˘.

duration

type	"AddBuff
*2(

stage_id±F

type"UnlockStage
?
	condition220

count

item_idÛ

type	"HasItem


idÑ

name"Event Rule 8É
¶
actionsö*ó
220

count

item_id˜

type	"AddItem
523

buff_id˙.

duration	

type	"AddBuff
*2(

stage_id≤F

type"UnlockStage
4
	condition'2%

level


type"LevelAtLeast


idÒÑ

name"Event Rule 9ä
¶
actionsö*ó
220

count

item_id¯

type	"AddItem
523

buff_id˚.

duration

type	"AddBuff
*2(

stage_id≥F

type"UnlockStage
:
	condition-2+

quest_idì'

type"QuestCompleted


idÚÑ

name"Event Rule 10è
¶
actionsö*ó
220

count

item_id˘

type	"AddItem
523

buff_id¸.

duration

type	"AddBuff
*2(

stage_id¥F

type"UnlockStage
?
	condition220

count

item_idˆ

type	"HasItem


idÛÑ

name"Event Rule 11Ñ
¶
actionsö*ó
220

count

item_id˙

type	"AddItem
523

buff_id˝.

duration

type	"AddBuff
*2(

stage_idµF

type"UnlockStage
4
	condition'2%

level

type"LevelAtLeast


idÙÑ

name"Event Rule 12ä
¶
actionsö*ó
220

count

item_id˚

type	"AddItem
523

buff_id˛.

duration

type	"AddBuff
*2(

stage_id∂F

type"UnlockStage
:
	condition-2+

quest_idñ'

type"QuestCompleted


idıÑ

name"Event Rule 13è
¶
actionsö*ó
220

count

item_id¸

type	"AddItem
523

buff_idˇ.

duration	

type	"AddBuff
*2(

stage_id∑F

type"UnlockStage
?
	condition220

count

item_id˘

type	"HasItem


idˆÑ

name"Event Rule 14Ñ
¶
actionsö*ó
220

count

item_id˝

type	"AddItem
523

buff_idÄ/

duration

type	"AddBuff
*2(

stage_id∏F

type"UnlockStage
4
	condition'2%

level

type"LevelAtLeast


id˜Ñ

name"Event Rule 15ä
¶
actionsö*ó
220

count

item_id˛

type	"AddItem
523

buff_idÅ/

duration

type	"AddBuff
*2(

stage_idπF

type"UnlockStage
:
	condition-2+

quest_idô'

type"QuestCompleted


id¯Ñ

name"Event Rule 16è
¶
actionsö*ó
220

count

item_idˇ

type	"AddItem
523

buff_idÇ/

duration

type	"AddBuff
*2(

stage_id∫F

type"UnlockStage
?
	condition220

count

item_id¸

type	"HasItem


id˘Ñ

name"Event Rule 17Ñ
¶
actionsö*ó
220

count

item_idÄ

type	"AddItem
523

buff_idÉ/

duration

type	"AddBuff
*2(

stage_idªF

type"UnlockStage
4
	condition'2%

level

type"LevelAtLeast


id˙Ñ

name"Event Rule 18ä
¶
actionsö*ó
220

count

item_idÅ

type	"AddItem
523

buff_idÑ/

duration	

type	"AddBuff
*2(

stage_idºF

type"UnlockStage
:
	condition-2+

quest_idú'

type"QuestCompleted


id˚Ñ

name"Event Rule 19è
¶
actionsö*ó
220

count

item_idÇ

type	"AddItem
523

buff_idÒ.

duration

type	"AddBuff
*2(

stage_idΩF

type"UnlockStage
?
	condition220

count

item_idˇ

type	"HasItem


id¸Ñ

name"Event Rule 20*Ù	
ComplexRuleü
∏
actions¨*©
220

count

item_idÈ

type	"AddItem
<2:

buff_idÒ.

duration	      >@

type	"AddBuff
523

action_group_id∑ç

type"RunActionGroup
Œ
budget√2¿
3
fixed*2(

countË

id 

kind"Gold
.
limits$*"
*
"quest

*
"daily

ÿ
randomÕ* 
g2e
2
cost*2(

count
	
idÈ

kind"Item
!
labels*
	"starter
"weapon

weightd
_2]
4
cost,2*

count

id 

kind	"Diamond

labels*
	"premium

weight


id—å

name"Nested launch rewards

root_action_groupµç
I
root_condition725

condition_group_idπî

type"AllConditions¬
a
actionsV*T
*2(

stage_id™F

type"UnlockStage
&2$

mail_idôu

type
"SendMail
Õ
budget¬2ø
3
fixed*2(

count
	
idÍ

kind"Item
-
limits#*!
*
"weekly

*
"vip

ÿ
randomÕ* 
^2\
2
cost*2(

countÙ

id 

kind"Gold

labels*

"currency

weight<
h2f
2
cost*2(

count
	
id—

kind"Item
"
labels*
"potion

"fallback

weight(


id“å

name"Quest chain unlock

root_action_group∂ç
H
root_condition624

condition_group_id∫î

type"AnyCondition*ﬁ
ComplexConditionGroupõ
m

conditions_*]
'2%

level

type"LevelAtLeast
220

count

item_idÈ

type	"HasItem


idπî

name"Veteran player gates¶
s

conditionse*c
-2+

quest_idä'

type"QuestCompleted
220

count

item_idÍ

type	"HasItem


id∫î
#
name"Quest and inventory gates*¨
ComplexConditionGroupEntry[

group_idπî


idùï
	
seq
0
value'2%

level

type"LevelAtLeastf

group_idπî


idûï
	
seq
;
value220

count

item_idÈ

type	"HasItema

group_id∫î


idüï
	
seq
6
value-2+

quest_idä'

type"QuestCompletedf

group_id∫î


id†ï
	
seq
;
value220

count

item_idÍ

type	"HasItem*◊
ComplexRuleCondition_


idÅñ

rule_id—å
@
value725

condition_group_idπî

type"AllConditions^


idÇñ

rule_id“å
?
value624

condition_group_id∫î

type"AnyCondition*¸
ComplexActionGroupÊ
∏
actions¨*©
220

count

item_idÈ

type	"AddItem
<2:

buff_idÒ.

duration	      >@

type	"AddBuff
523

action_group_id∑ç

type"RunActionGroup


idµç

name"Launch reward chainç
a
actionsV*T
*2(

stage_id™F

type"UnlockStage
&2$

mail_idôu

type
"SendMail


id∂ç

name"Quest unlock chainm
A
actions6*4
220

count

item_id—

type	"AddItem


id∑ç

name"Nested bonus group*˝
ComplexActionEntryf

group_idµç


idôé
	
seq
;
value220

count

item_idÈ

type	"AddItemp

group_idµç


idöé
	
seq
E
value<2:

buff_idÒ.

duration	      >@

type	"AddBuffi

group_idµç


idõé
	
seq
>
value523

action_group_id∑ç

type"RunActionGroup^

group_id∂ç


idúé
	
seq
3
value*2(

stage_id™F

type"UnlockStageZ

group_id∂ç


idùé
	
seq
/
value&2$

mail_idôu

type
"SendMailf

group_id∑ç


idûé
	
seq
;
value220

count

item_id—

type	"AddItem24ae91b3892293cc4:68de214b62c05454