package com.sora.showcase;

import java.util.Collection;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Function;

enum SoraTableMode {
    LIST,
    MAP,
    SINGLETON,
}

interface SoraTable {
    String name();
    SoraTableMode mode();
    String key();
    String rowType();
    int size();
}

final class ItemTable implements SoraTable {
    private final java.util.Map<Integer, Item> rows;

    private ItemTable(java.util.Map<Integer, Item> rows) {
        this.rows = rows;
    }

    static ItemTable decode(SoraBundle bundle) {
        return new ItemTable(SoraConfig.decodeMapTable(bundle.decodeTable("Item", Item::decode), row -> row.id));
    }

    public java.util.Map<Integer, Item> rows() {
        return rows;
    }
    public Item get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Item";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Item";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class SkillTable implements SoraTable {
    private final java.util.Map<Integer, Skill> rows;

    private SkillTable(java.util.Map<Integer, Skill> rows) {
        this.rows = rows;
    }

    static SkillTable decode(SoraBundle bundle) {
        return new SkillTable(SoraConfig.decodeMapTable(bundle.decodeTable("Skill", Skill::decode), row -> row.id));
    }

    public java.util.Map<Integer, Skill> rows() {
        return rows;
    }
    public Skill get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Skill";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Skill";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class QuestTable implements SoraTable {
    private final java.util.Map<Integer, Quest> rows;

    private QuestTable(java.util.Map<Integer, Quest> rows) {
        this.rows = rows;
    }

    static QuestTable decode(SoraBundle bundle) {
        return new QuestTable(SoraConfig.decodeMapTable(bundle.decodeTable("Quest", Quest::decode), row -> row.id));
    }

    public java.util.Map<Integer, Quest> rows() {
        return rows;
    }
    public Quest get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Quest";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Quest";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class QuestRewardTable implements SoraTable {
    private final java.util.List<QuestReward> rows;

    private QuestRewardTable(java.util.List<QuestReward> rows) {
        this.rows = rows;
    }

    static QuestRewardTable decode(SoraBundle bundle) {
        return new QuestRewardTable(bundle.decodeTable("QuestReward", QuestReward::decode));
    }

    public java.util.List<QuestReward> rows() {
        return rows;
    }
    @Override
    public String name() {
        return "QuestReward";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.LIST;
    }

    @Override
    public String key() {
        return null;
    }

    @Override
    public String rowType() {
        return "QuestReward";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class GameSettingsTable implements SoraTable {
    private final GameSettings rows;

    private GameSettingsTable(GameSettings rows) {
        this.rows = rows;
    }

    static GameSettingsTable decode(SoraBundle bundle) {
        return new GameSettingsTable(SoraConfig.requireSingletonTable(bundle.decodeTable("GameSettings", GameSettings::decode), "GameSettings"));
    }

    public GameSettings rows() {
        return rows;
    }
    @Override
    public String name() {
        return "GameSettings";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.SINGLETON;
    }

    @Override
    public String key() {
        return null;
    }

    @Override
    public String rowType() {
        return "GameSettings";
    }

    @Override
    public int size() {
        return 1;
    }
}

final class LocalizationTable implements SoraTable {
    private final java.util.Map<String, Localization> rows;

    private LocalizationTable(java.util.Map<String, Localization> rows) {
        this.rows = rows;
    }

    static LocalizationTable decode(SoraBundle bundle) {
        return new LocalizationTable(SoraConfig.decodeMapTable(bundle.decodeTable("Localization", Localization::decode), row -> row.key));
    }

    public java.util.Map<String, Localization> rows() {
        return rows;
    }
    public Localization get(String key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Localization";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "key";
    }

    @Override
    public String rowType() {
        return "Localization";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class LevelExpTable implements SoraTable {
    private final java.util.Map<Integer, LevelExp> rows;

    private LevelExpTable(java.util.Map<Integer, LevelExp> rows) {
        this.rows = rows;
    }

    static LevelExpTable decode(SoraBundle bundle) {
        return new LevelExpTable(SoraConfig.decodeMapTable(bundle.decodeTable("LevelExp", LevelExp::decode), row -> row.level));
    }

    public java.util.Map<Integer, LevelExp> rows() {
        return rows;
    }
    public LevelExp get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "LevelExp";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "level";
    }

    @Override
    public String rowType() {
        return "LevelExp";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class CharacterTable implements SoraTable {
    private final java.util.Map<Integer, Character> rows;

    private CharacterTable(java.util.Map<Integer, Character> rows) {
        this.rows = rows;
    }

    static CharacterTable decode(SoraBundle bundle) {
        return new CharacterTable(SoraConfig.decodeMapTable(bundle.decodeTable("Character", Character::decode), row -> row.id));
    }

    public java.util.Map<Integer, Character> rows() {
        return rows;
    }
    public Character get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Character";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Character";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class CharacterSkillTable implements SoraTable {
    private final java.util.List<CharacterSkill> rows;

    private CharacterSkillTable(java.util.List<CharacterSkill> rows) {
        this.rows = rows;
    }

    static CharacterSkillTable decode(SoraBundle bundle) {
        return new CharacterSkillTable(bundle.decodeTable("CharacterSkill", CharacterSkill::decode));
    }

    public java.util.List<CharacterSkill> rows() {
        return rows;
    }
    @Override
    public String name() {
        return "CharacterSkill";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.LIST;
    }

    @Override
    public String key() {
        return null;
    }

    @Override
    public String rowType() {
        return "CharacterSkill";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class BuffTable implements SoraTable {
    private final java.util.Map<Integer, Buff> rows;

    private BuffTable(java.util.Map<Integer, Buff> rows) {
        this.rows = rows;
    }

    static BuffTable decode(SoraBundle bundle) {
        return new BuffTable(SoraConfig.decodeMapTable(bundle.decodeTable("Buff", Buff::decode), row -> row.id));
    }

    public java.util.Map<Integer, Buff> rows() {
        return rows;
    }
    public Buff get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Buff";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Buff";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class DropGroupTable implements SoraTable {
    private final java.util.Map<Integer, DropGroup> rows;

    private DropGroupTable(java.util.Map<Integer, DropGroup> rows) {
        this.rows = rows;
    }

    static DropGroupTable decode(SoraBundle bundle) {
        return new DropGroupTable(SoraConfig.decodeMapTable(bundle.decodeTable("DropGroup", DropGroup::decode), row -> row.id));
    }

    public java.util.Map<Integer, DropGroup> rows() {
        return rows;
    }
    public DropGroup get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "DropGroup";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "DropGroup";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class DropEntryTable implements SoraTable {
    private final java.util.List<DropEntry> rows;

    private DropEntryTable(java.util.List<DropEntry> rows) {
        this.rows = rows;
    }

    static DropEntryTable decode(SoraBundle bundle) {
        return new DropEntryTable(bundle.decodeTable("DropEntry", DropEntry::decode));
    }

    public java.util.List<DropEntry> rows() {
        return rows;
    }
    @Override
    public String name() {
        return "DropEntry";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.LIST;
    }

    @Override
    public String key() {
        return null;
    }

    @Override
    public String rowType() {
        return "DropEntry";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class MonsterTable implements SoraTable {
    private final java.util.Map<Integer, Monster> rows;

    private MonsterTable(java.util.Map<Integer, Monster> rows) {
        this.rows = rows;
    }

    static MonsterTable decode(SoraBundle bundle) {
        return new MonsterTable(SoraConfig.decodeMapTable(bundle.decodeTable("Monster", Monster::decode), row -> row.id));
    }

    public java.util.Map<Integer, Monster> rows() {
        return rows;
    }
    public Monster get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Monster";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Monster";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class StageTable implements SoraTable {
    private final java.util.Map<Integer, Stage> rows;

    private StageTable(java.util.Map<Integer, Stage> rows) {
        this.rows = rows;
    }

    static StageTable decode(SoraBundle bundle) {
        return new StageTable(SoraConfig.decodeMapTable(bundle.decodeTable("Stage", Stage::decode), row -> row.id));
    }

    public java.util.Map<Integer, Stage> rows() {
        return rows;
    }
    public Stage get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Stage";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Stage";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class StageRewardTable implements SoraTable {
    private final java.util.List<StageReward> rows;

    private StageRewardTable(java.util.List<StageReward> rows) {
        this.rows = rows;
    }

    static StageRewardTable decode(SoraBundle bundle) {
        return new StageRewardTable(bundle.decodeTable("StageReward", StageReward::decode));
    }

    public java.util.List<StageReward> rows() {
        return rows;
    }
    @Override
    public String name() {
        return "StageReward";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.LIST;
    }

    @Override
    public String key() {
        return null;
    }

    @Override
    public String rowType() {
        return "StageReward";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class DungeonTable implements SoraTable {
    private final java.util.Map<Integer, Dungeon> rows;

    private DungeonTable(java.util.Map<Integer, Dungeon> rows) {
        this.rows = rows;
    }

    static DungeonTable decode(SoraBundle bundle) {
        return new DungeonTable(SoraConfig.decodeMapTable(bundle.decodeTable("Dungeon", Dungeon::decode), row -> row.id));
    }

    public java.util.Map<Integer, Dungeon> rows() {
        return rows;
    }
    public Dungeon get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Dungeon";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Dungeon";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class ShopTable implements SoraTable {
    private final java.util.Map<Integer, Shop> rows;

    private ShopTable(java.util.Map<Integer, Shop> rows) {
        this.rows = rows;
    }

    static ShopTable decode(SoraBundle bundle) {
        return new ShopTable(SoraConfig.decodeMapTable(bundle.decodeTable("Shop", Shop::decode), row -> row.id));
    }

    public java.util.Map<Integer, Shop> rows() {
        return rows;
    }
    public Shop get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Shop";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Shop";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class ShopItemTable implements SoraTable {
    private final java.util.List<ShopItem> rows;

    private ShopItemTable(java.util.List<ShopItem> rows) {
        this.rows = rows;
    }

    static ShopItemTable decode(SoraBundle bundle) {
        return new ShopItemTable(bundle.decodeTable("ShopItem", ShopItem::decode));
    }

    public java.util.List<ShopItem> rows() {
        return rows;
    }
    @Override
    public String name() {
        return "ShopItem";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.LIST;
    }

    @Override
    public String key() {
        return null;
    }

    @Override
    public String rowType() {
        return "ShopItem";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class RecipeTable implements SoraTable {
    private final java.util.Map<Integer, Recipe> rows;

    private RecipeTable(java.util.Map<Integer, Recipe> rows) {
        this.rows = rows;
    }

    static RecipeTable decode(SoraBundle bundle) {
        return new RecipeTable(SoraConfig.decodeMapTable(bundle.decodeTable("Recipe", Recipe::decode), row -> row.id));
    }

    public java.util.Map<Integer, Recipe> rows() {
        return rows;
    }
    public Recipe get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Recipe";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Recipe";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class GachaPoolTable implements SoraTable {
    private final java.util.Map<Integer, GachaPool> rows;

    private GachaPoolTable(java.util.Map<Integer, GachaPool> rows) {
        this.rows = rows;
    }

    static GachaPoolTable decode(SoraBundle bundle) {
        return new GachaPoolTable(SoraConfig.decodeMapTable(bundle.decodeTable("GachaPool", GachaPool::decode), row -> row.id));
    }

    public java.util.Map<Integer, GachaPool> rows() {
        return rows;
    }
    public GachaPool get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "GachaPool";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "GachaPool";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class GachaItemTable implements SoraTable {
    private final java.util.List<GachaItem> rows;

    private GachaItemTable(java.util.List<GachaItem> rows) {
        this.rows = rows;
    }

    static GachaItemTable decode(SoraBundle bundle) {
        return new GachaItemTable(bundle.decodeTable("GachaItem", GachaItem::decode));
    }

    public java.util.List<GachaItem> rows() {
        return rows;
    }
    @Override
    public String name() {
        return "GachaItem";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.LIST;
    }

    @Override
    public String key() {
        return null;
    }

    @Override
    public String rowType() {
        return "GachaItem";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class EquipmentSetTable implements SoraTable {
    private final java.util.Map<Integer, EquipmentSet> rows;

    private EquipmentSetTable(java.util.Map<Integer, EquipmentSet> rows) {
        this.rows = rows;
    }

    static EquipmentSetTable decode(SoraBundle bundle) {
        return new EquipmentSetTable(SoraConfig.decodeMapTable(bundle.decodeTable("EquipmentSet", EquipmentSet::decode), row -> row.id));
    }

    public java.util.Map<Integer, EquipmentSet> rows() {
        return rows;
    }
    public EquipmentSet get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "EquipmentSet";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "EquipmentSet";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class AchievementTable implements SoraTable {
    private final java.util.Map<Integer, Achievement> rows;

    private AchievementTable(java.util.Map<Integer, Achievement> rows) {
        this.rows = rows;
    }

    static AchievementTable decode(SoraBundle bundle) {
        return new AchievementTable(SoraConfig.decodeMapTable(bundle.decodeTable("Achievement", Achievement::decode), row -> row.id));
    }

    public java.util.Map<Integer, Achievement> rows() {
        return rows;
    }
    public Achievement get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Achievement";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Achievement";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class VipLevelTable implements SoraTable {
    private final java.util.Map<Integer, VipLevel> rows;

    private VipLevelTable(java.util.Map<Integer, VipLevel> rows) {
        this.rows = rows;
    }

    static VipLevelTable decode(SoraBundle bundle) {
        return new VipLevelTable(SoraConfig.decodeMapTable(bundle.decodeTable("VipLevel", VipLevel::decode), row -> row.level));
    }

    public java.util.Map<Integer, VipLevel> rows() {
        return rows;
    }
    public VipLevel get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "VipLevel";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "level";
    }

    @Override
    public String rowType() {
        return "VipLevel";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class MailTemplateTable implements SoraTable {
    private final java.util.Map<Integer, MailTemplate> rows;

    private MailTemplateTable(java.util.Map<Integer, MailTemplate> rows) {
        this.rows = rows;
    }

    static MailTemplateTable decode(SoraBundle bundle) {
        return new MailTemplateTable(SoraConfig.decodeMapTable(bundle.decodeTable("MailTemplate", MailTemplate::decode), row -> row.id));
    }

    public java.util.Map<Integer, MailTemplate> rows() {
        return rows;
    }
    public MailTemplate get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "MailTemplate";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "MailTemplate";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class MailRewardTable implements SoraTable {
    private final java.util.List<MailReward> rows;

    private MailRewardTable(java.util.List<MailReward> rows) {
        this.rows = rows;
    }

    static MailRewardTable decode(SoraBundle bundle) {
        return new MailRewardTable(bundle.decodeTable("MailReward", MailReward::decode));
    }

    public java.util.List<MailReward> rows() {
        return rows;
    }
    @Override
    public String name() {
        return "MailReward";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.LIST;
    }

    @Override
    public String key() {
        return null;
    }

    @Override
    public String rowType() {
        return "MailReward";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class DialogueTable implements SoraTable {
    private final java.util.Map<Integer, Dialogue> rows;

    private DialogueTable(java.util.Map<Integer, Dialogue> rows) {
        this.rows = rows;
    }

    static DialogueTable decode(SoraBundle bundle) {
        return new DialogueTable(SoraConfig.decodeMapTable(bundle.decodeTable("Dialogue", Dialogue::decode), row -> row.id));
    }

    public java.util.Map<Integer, Dialogue> rows() {
        return rows;
    }
    public Dialogue get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "Dialogue";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "Dialogue";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

final class EventRuleTable implements SoraTable {
    private final java.util.Map<Integer, EventRule> rows;

    private EventRuleTable(java.util.Map<Integer, EventRule> rows) {
        this.rows = rows;
    }

    static EventRuleTable decode(SoraBundle bundle) {
        return new EventRuleTable(SoraConfig.decodeMapTable(bundle.decodeTable("EventRule", EventRule::decode), row -> row.id));
    }

    public java.util.Map<Integer, EventRule> rows() {
        return rows;
    }
    public EventRule get(Integer key) {
        return rows.get(key);
    }
    @Override
    public String name() {
        return "EventRule";
    }

    @Override
    public SoraTableMode mode() {
        return SoraTableMode.MAP;
    }

    @Override
    public String key() {
        return "id";
    }

    @Override
    public String rowType() {
        return "EventRule";
    }

    @Override
    public int size() {
        return rows.size();
    }
}

public final class SoraConfig {
    private final Map<String, SoraTable> tables;

    private SoraConfig(Map<String, SoraTable> tables) {
        this.tables = tables;
    }

    public static SoraConfig fromBytes(byte[] bytes) {
        var bundle = SoraBundle.parse(bytes);
        var tables = new HashMap<String, SoraTable>(28);
        tables.put("Item", ItemTable.decode(bundle));
        tables.put("Skill", SkillTable.decode(bundle));
        tables.put("Quest", QuestTable.decode(bundle));
        tables.put("QuestReward", QuestRewardTable.decode(bundle));
        tables.put("GameSettings", GameSettingsTable.decode(bundle));
        tables.put("Localization", LocalizationTable.decode(bundle));
        tables.put("LevelExp", LevelExpTable.decode(bundle));
        tables.put("Character", CharacterTable.decode(bundle));
        tables.put("CharacterSkill", CharacterSkillTable.decode(bundle));
        tables.put("Buff", BuffTable.decode(bundle));
        tables.put("DropGroup", DropGroupTable.decode(bundle));
        tables.put("DropEntry", DropEntryTable.decode(bundle));
        tables.put("Monster", MonsterTable.decode(bundle));
        tables.put("Stage", StageTable.decode(bundle));
        tables.put("StageReward", StageRewardTable.decode(bundle));
        tables.put("Dungeon", DungeonTable.decode(bundle));
        tables.put("Shop", ShopTable.decode(bundle));
        tables.put("ShopItem", ShopItemTable.decode(bundle));
        tables.put("Recipe", RecipeTable.decode(bundle));
        tables.put("GachaPool", GachaPoolTable.decode(bundle));
        tables.put("GachaItem", GachaItemTable.decode(bundle));
        tables.put("EquipmentSet", EquipmentSetTable.decode(bundle));
        tables.put("Achievement", AchievementTable.decode(bundle));
        tables.put("VipLevel", VipLevelTable.decode(bundle));
        tables.put("MailTemplate", MailTemplateTable.decode(bundle));
        tables.put("MailReward", MailRewardTable.decode(bundle));
        tables.put("Dialogue", DialogueTable.decode(bundle));
        tables.put("EventRule", EventRuleTable.decode(bundle));
        return new SoraConfig(tables);
    }

    public Collection<SoraTable> tables() {
        return tables.values();
    }

    private <T extends SoraTable> T table(String name, Class<T> type) {
        var table = tables.get(name);
        if (type.isInstance(table)) {
            return type.cast(table);
        }
        throw new SoraReadException("generated SoraConfig is missing table `" + name + "` or has an unexpected table type");
    }
    public ItemTable item() {
        return table("Item", ItemTable.class);
    }
    public SkillTable skill() {
        return table("Skill", SkillTable.class);
    }
    public QuestTable quest() {
        return table("Quest", QuestTable.class);
    }
    public QuestRewardTable questReward() {
        return table("QuestReward", QuestRewardTable.class);
    }
    public GameSettingsTable gameSettings() {
        return table("GameSettings", GameSettingsTable.class);
    }
    public LocalizationTable localization() {
        return table("Localization", LocalizationTable.class);
    }
    public LevelExpTable levelExp() {
        return table("LevelExp", LevelExpTable.class);
    }
    public CharacterTable character() {
        return table("Character", CharacterTable.class);
    }
    public CharacterSkillTable characterSkill() {
        return table("CharacterSkill", CharacterSkillTable.class);
    }
    public BuffTable buff() {
        return table("Buff", BuffTable.class);
    }
    public DropGroupTable dropGroup() {
        return table("DropGroup", DropGroupTable.class);
    }
    public DropEntryTable dropEntry() {
        return table("DropEntry", DropEntryTable.class);
    }
    public MonsterTable monster() {
        return table("Monster", MonsterTable.class);
    }
    public StageTable stage() {
        return table("Stage", StageTable.class);
    }
    public StageRewardTable stageReward() {
        return table("StageReward", StageRewardTable.class);
    }
    public DungeonTable dungeon() {
        return table("Dungeon", DungeonTable.class);
    }
    public ShopTable shop() {
        return table("Shop", ShopTable.class);
    }
    public ShopItemTable shopItem() {
        return table("ShopItem", ShopItemTable.class);
    }
    public RecipeTable recipe() {
        return table("Recipe", RecipeTable.class);
    }
    public GachaPoolTable gachaPool() {
        return table("GachaPool", GachaPoolTable.class);
    }
    public GachaItemTable gachaItem() {
        return table("GachaItem", GachaItemTable.class);
    }
    public EquipmentSetTable equipmentSet() {
        return table("EquipmentSet", EquipmentSetTable.class);
    }
    public AchievementTable achievement() {
        return table("Achievement", AchievementTable.class);
    }
    public VipLevelTable vipLevel() {
        return table("VipLevel", VipLevelTable.class);
    }
    public MailTemplateTable mailTemplate() {
        return table("MailTemplate", MailTemplateTable.class);
    }
    public MailRewardTable mailReward() {
        return table("MailReward", MailRewardTable.class);
    }
    public DialogueTable dialogue() {
        return table("Dialogue", DialogueTable.class);
    }
    public EventRuleTable eventRule() {
        return table("EventRule", EventRuleTable.class);
    }
    static <K, V> Map<K, V> decodeMapTable(List<V> rows, Function<V, K> key) {
        var map = new HashMap<K, V>(rows.size());
        for (var row : rows) {
            map.put(key.apply(row), row);
        }
        return map;
    }

    static <T> T requireSingletonTable(List<T> rows, String name) {
        if (rows.size() != 1) {
            throw new SoraReadException("expected singleton table `" + name + "` to contain exactly 1 row, got " + rows.size());
        }
        return rows.get(0);
    }
}