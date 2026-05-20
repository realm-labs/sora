package game_config_showcase

data class SoraConfig(
    val item: Map<Int, Item>,
    val skill: Map<Int, Skill>,
    val quest: Map<Int, Quest>,
    val quest_reward: List<QuestReward>,
    val game_settings: GameSettings,
    val localization: Map<String, Localization>,
    val level_exp: Map<Int, LevelExp>,
    val character: Map<Int, Character>,
    val character_skill: List<CharacterSkill>,
    val buff: Map<Int, Buff>,
    val drop_group: Map<Int, DropGroup>,
    val drop_entry: List<DropEntry>,
    val monster: Map<Int, Monster>,
    val stage: Map<Int, Stage>,
    val stage_reward: List<StageReward>,
    val dungeon: Map<Int, Dungeon>,
    val shop: Map<Int, Shop>,
    val shop_item: List<ShopItem>,
    val recipe: Map<Int, Recipe>,
    val gacha_pool: Map<Int, GachaPool>,
    val gacha_item: List<GachaItem>,
    val equipment_set: Map<Int, EquipmentSet>,
    val achievement: Map<Int, Achievement>,
    val vip_level: Map<Int, VipLevel>,
    val mail_template: Map<Int, MailTemplate>,
    val mail_reward: List<MailReward>,
    val dialogue: Map<Int, Dialogue>,
    val event_rule: Map<Int, EventRule>,
) {
    fun getItem(key: Int): Item? = item[key]

    fun itemValues(): Collection<Item> = item.values
    fun getSkill(key: Int): Skill? = skill[key]

    fun skillValues(): Collection<Skill> = skill.values
    fun getQuest(key: Int): Quest? = quest[key]

    fun questValues(): Collection<Quest> = quest.values
    fun questRewardRows(): List<QuestReward> = quest_reward
    fun gameSettingsRow(): GameSettings = game_settings
    fun getLocalization(key: String): Localization? = localization[key]

    fun localizationValues(): Collection<Localization> = localization.values
    fun getLevelExp(key: Int): LevelExp? = level_exp[key]

    fun levelExpValues(): Collection<LevelExp> = level_exp.values
    fun getCharacter(key: Int): Character? = character[key]

    fun characterValues(): Collection<Character> = character.values
    fun characterSkillRows(): List<CharacterSkill> = character_skill
    fun getBuff(key: Int): Buff? = buff[key]

    fun buffValues(): Collection<Buff> = buff.values
    fun getDropGroup(key: Int): DropGroup? = drop_group[key]

    fun dropGroupValues(): Collection<DropGroup> = drop_group.values
    fun dropEntryRows(): List<DropEntry> = drop_entry
    fun getMonster(key: Int): Monster? = monster[key]

    fun monsterValues(): Collection<Monster> = monster.values
    fun getStage(key: Int): Stage? = stage[key]

    fun stageValues(): Collection<Stage> = stage.values
    fun stageRewardRows(): List<StageReward> = stage_reward
    fun getDungeon(key: Int): Dungeon? = dungeon[key]

    fun dungeonValues(): Collection<Dungeon> = dungeon.values
    fun getShop(key: Int): Shop? = shop[key]

    fun shopValues(): Collection<Shop> = shop.values
    fun shopItemRows(): List<ShopItem> = shop_item
    fun getRecipe(key: Int): Recipe? = recipe[key]

    fun recipeValues(): Collection<Recipe> = recipe.values
    fun getGachaPool(key: Int): GachaPool? = gacha_pool[key]

    fun gachaPoolValues(): Collection<GachaPool> = gacha_pool.values
    fun gachaItemRows(): List<GachaItem> = gacha_item
    fun getEquipmentSet(key: Int): EquipmentSet? = equipment_set[key]

    fun equipmentSetValues(): Collection<EquipmentSet> = equipment_set.values
    fun getAchievement(key: Int): Achievement? = achievement[key]

    fun achievementValues(): Collection<Achievement> = achievement.values
    fun getVipLevel(key: Int): VipLevel? = vip_level[key]

    fun vipLevelValues(): Collection<VipLevel> = vip_level.values
    fun getMailTemplate(key: Int): MailTemplate? = mail_template[key]

    fun mailTemplateValues(): Collection<MailTemplate> = mail_template.values
    fun mailRewardRows(): List<MailReward> = mail_reward
    fun getDialogue(key: Int): Dialogue? = dialogue[key]

    fun dialogueValues(): Collection<Dialogue> = dialogue.values
    fun getEventRule(key: Int): EventRule? = event_rule[key]

    fun eventRuleValues(): Collection<EventRule> = event_rule.values

    companion object {
        fun fromBytes(bytes: ByteArray): SoraConfig {
            val bundle = SoraBundle.parse(bytes)
            return SoraConfig(
                item = bundle.decodeTable("Item", Item::decode).associateBy { it.id },
                skill = bundle.decodeTable("Skill", Skill::decode).associateBy { it.id },
                quest = bundle.decodeTable("Quest", Quest::decode).associateBy { it.id },
                quest_reward = bundle.decodeTable("QuestReward", QuestReward::decode),
                game_settings = requireSingletonTable(bundle.decodeTable("GameSettings", GameSettings::decode), "GameSettings"),
                localization = bundle.decodeTable("Localization", Localization::decode).associateBy { it.key },
                level_exp = bundle.decodeTable("LevelExp", LevelExp::decode).associateBy { it.level },
                character = bundle.decodeTable("Character", Character::decode).associateBy { it.id },
                character_skill = bundle.decodeTable("CharacterSkill", CharacterSkill::decode),
                buff = bundle.decodeTable("Buff", Buff::decode).associateBy { it.id },
                drop_group = bundle.decodeTable("DropGroup", DropGroup::decode).associateBy { it.id },
                drop_entry = bundle.decodeTable("DropEntry", DropEntry::decode),
                monster = bundle.decodeTable("Monster", Monster::decode).associateBy { it.id },
                stage = bundle.decodeTable("Stage", Stage::decode).associateBy { it.id },
                stage_reward = bundle.decodeTable("StageReward", StageReward::decode),
                dungeon = bundle.decodeTable("Dungeon", Dungeon::decode).associateBy { it.id },
                shop = bundle.decodeTable("Shop", Shop::decode).associateBy { it.id },
                shop_item = bundle.decodeTable("ShopItem", ShopItem::decode),
                recipe = bundle.decodeTable("Recipe", Recipe::decode).associateBy { it.id },
                gacha_pool = bundle.decodeTable("GachaPool", GachaPool::decode).associateBy { it.id },
                gacha_item = bundle.decodeTable("GachaItem", GachaItem::decode),
                equipment_set = bundle.decodeTable("EquipmentSet", EquipmentSet::decode).associateBy { it.id },
                achievement = bundle.decodeTable("Achievement", Achievement::decode).associateBy { it.id },
                vip_level = bundle.decodeTable("VipLevel", VipLevel::decode).associateBy { it.level },
                mail_template = bundle.decodeTable("MailTemplate", MailTemplate::decode).associateBy { it.id },
                mail_reward = bundle.decodeTable("MailReward", MailReward::decode),
                dialogue = bundle.decodeTable("Dialogue", Dialogue::decode).associateBy { it.id },
                event_rule = bundle.decodeTable("EventRule", EventRule::decode).associateBy { it.id },
            )
        }
    }
}