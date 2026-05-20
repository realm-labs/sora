package game_config_showcase

data class SoraConfig(
    val item: Map<Int, Item>,
    val skill: Map<Int, Skill>,
    val quest: Map<Int, Quest>,
    val quest_reward: List<QuestReward>,
    val game_settings: GameSettings,
) {
    fun getItem(key: Int): Item? = item[key]

    fun itemValues(): Collection<Item> = item.values
    fun getSkill(key: Int): Skill? = skill[key]

    fun skillValues(): Collection<Skill> = skill.values
    fun getQuest(key: Int): Quest? = quest[key]

    fun questValues(): Collection<Quest> = quest.values
    fun questRewardRows(): List<QuestReward> = quest_reward
    fun gameSettingsRow(): GameSettings = game_settings

    companion object {
        fun fromBytes(bytes: ByteArray): SoraConfig {
            val bundle = SoraBundle.parse(bytes)
            return SoraConfig(
                item = bundle.decodeTable("Item", Item::decode).associateBy { it.id },
                skill = bundle.decodeTable("Skill", Skill::decode).associateBy { it.id },
                quest = bundle.decodeTable("Quest", Quest::decode).associateBy { it.id },
                quest_reward = bundle.decodeTable("QuestReward", QuestReward::decode),
                game_settings = requireSingletonTable(bundle.decodeTable("GameSettings", GameSettings::decode), "GameSettings"),
            )
        }
    }
}