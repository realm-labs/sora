#nullable enable

using System;
using System.Collections.Generic;
using System.Linq;

namespace com.sora.showcase;

public enum SoraTableMode
{
    List,
    Map,
    Singleton,
}

public interface ISoraTable
{
    string Name { get; }
    SoraTableMode Mode { get; }
    string? Key { get; }
    string RowType { get; }
    int Count { get; }
}

public sealed class ItemTable : ISoraTable
{
    private readonly Dictionary<int, Item> rows;
    private readonly Dictionary<string, Item> byName;
    private readonly Dictionary<ItemType, List<Item>> byItemType;

    internal ItemTable(Dictionary<int, Item> rows, Dictionary<string, Item> byName, Dictionary<ItemType, List<Item>> byItemType)
    {
        this.rows = rows;
        this.byName = byName;
        this.byItemType = byItemType;
    }

    internal static ItemTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Item>("Item", Item.Decode));
    }

    internal static ItemTable FromRows(List<Item> rows)
    {
        return new ItemTable(
            SoraConfig.DecodeMapTable(rows, row => row.Id),
            SoraConfig.DecodeUniqueIndex(rows, row => row.Name),
            SoraConfig.DecodeIndex(rows, row => row.ItemType)
        );
    }

    public Dictionary<int, Item> Rows => rows;
    public Item? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public Item? GetByName(string name)
    {
        return byName.TryGetValue(name, out var row) ? row : default;
    }
    public IReadOnlyList<Item> FindByItemType(ItemType itemType)
    {
        return byItemType.TryGetValue(itemType, out var rows) ? rows : Array.Empty<Item>();
    }
    public string Name => "Item";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Item";
    public int Count => rows.Count;
}

public sealed class SkillTable : ISoraTable
{
    private readonly Dictionary<int, Skill> rows;

    internal SkillTable(Dictionary<int, Skill> rows)
    {
        this.rows = rows;
    }

    internal static SkillTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Skill>("Skill", Skill.Decode));
    }

    internal static SkillTable FromRows(List<Skill> rows)
    {
        return new SkillTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Skill> Rows => rows;
    public Skill? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Skill";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Skill";
    public int Count => rows.Count;
}

public sealed class QuestTable : ISoraTable
{
    private readonly Dictionary<int, Quest> rows;

    internal QuestTable(Dictionary<int, Quest> rows)
    {
        this.rows = rows;
    }

    internal static QuestTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Quest>("Quest", Quest.Decode));
    }

    internal static QuestTable FromRows(List<Quest> rows)
    {
        return new QuestTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Quest> Rows => rows;
    public Quest? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Quest";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Quest";
    public int Count => rows.Count;
}

public sealed class QuestRewardTable : ISoraTable
{
    private readonly List<QuestReward> rows;

    internal QuestRewardTable(List<QuestReward> rows)
    {
        this.rows = rows;
    }

    internal static QuestRewardTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<QuestReward>("QuestReward", QuestReward.Decode));
    }

    internal static QuestRewardTable FromRows(List<QuestReward> rows)
    {
        return new QuestRewardTable(rows);
    }

    public List<QuestReward> Rows => rows;
    public string Name => "QuestReward";
    public SoraTableMode Mode => SoraTableMode.List;
    public string? Key => null;
    public string RowType => "QuestReward";
    public int Count => rows.Count;
}

public sealed class GameSettingsTable : ISoraTable
{
    private readonly GameSettings rows;

    internal GameSettingsTable(GameSettings rows)
    {
        this.rows = rows;
    }

    internal static GameSettingsTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<GameSettings>("GameSettings", GameSettings.Decode));
    }

    internal static GameSettingsTable FromRows(List<GameSettings> rows)
    {
        return new GameSettingsTable(SoraConfig.RequireSingletonTable(rows, "GameSettings"));
    }

    public GameSettings Rows => rows;
    public string Name => "GameSettings";
    public SoraTableMode Mode => SoraTableMode.Singleton;
    public string? Key => null;
    public string RowType => "GameSettings";
    public int Count => 1;
}

public sealed class LocalizationTable : ISoraTable
{
    private readonly Dictionary<string, Localization> rows;

    internal LocalizationTable(Dictionary<string, Localization> rows)
    {
        this.rows = rows;
    }

    internal static LocalizationTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Localization>("Localization", Localization.Decode));
    }

    internal static LocalizationTable FromRows(List<Localization> rows)
    {
        return new LocalizationTable(SoraConfig.DecodeMapTable(rows, row => row.Key));
    }

    public Dictionary<string, Localization> Rows => rows;
    public Localization? Get(string key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Localization";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "key";
    public string RowType => "Localization";
    public int Count => rows.Count;
}

public sealed class LevelExpTable : ISoraTable
{
    private readonly Dictionary<int, LevelExp> rows;

    internal LevelExpTable(Dictionary<int, LevelExp> rows)
    {
        this.rows = rows;
    }

    internal static LevelExpTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<LevelExp>("LevelExp", LevelExp.Decode));
    }

    internal static LevelExpTable FromRows(List<LevelExp> rows)
    {
        return new LevelExpTable(SoraConfig.DecodeMapTable(rows, row => row.Level));
    }

    public Dictionary<int, LevelExp> Rows => rows;
    public LevelExp? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "LevelExp";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "level";
    public string RowType => "LevelExp";
    public int Count => rows.Count;
}

public sealed class CharacterTable : ISoraTable
{
    private readonly Dictionary<int, Character> rows;

    internal CharacterTable(Dictionary<int, Character> rows)
    {
        this.rows = rows;
    }

    internal static CharacterTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Character>("Character", Character.Decode));
    }

    internal static CharacterTable FromRows(List<Character> rows)
    {
        return new CharacterTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Character> Rows => rows;
    public Character? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Character";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Character";
    public int Count => rows.Count;
}

public sealed class CharacterSkillTable : ISoraTable
{
    private readonly List<CharacterSkill> rows;

    internal CharacterSkillTable(List<CharacterSkill> rows)
    {
        this.rows = rows;
    }

    internal static CharacterSkillTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<CharacterSkill>("CharacterSkill", CharacterSkill.Decode));
    }

    internal static CharacterSkillTable FromRows(List<CharacterSkill> rows)
    {
        return new CharacterSkillTable(rows);
    }

    public List<CharacterSkill> Rows => rows;
    public string Name => "CharacterSkill";
    public SoraTableMode Mode => SoraTableMode.List;
    public string? Key => null;
    public string RowType => "CharacterSkill";
    public int Count => rows.Count;
}

public sealed class BuffTable : ISoraTable
{
    private readonly Dictionary<int, Buff> rows;

    internal BuffTable(Dictionary<int, Buff> rows)
    {
        this.rows = rows;
    }

    internal static BuffTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Buff>("Buff", Buff.Decode));
    }

    internal static BuffTable FromRows(List<Buff> rows)
    {
        return new BuffTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Buff> Rows => rows;
    public Buff? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Buff";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Buff";
    public int Count => rows.Count;
}

public sealed class DropGroupTable : ISoraTable
{
    private readonly Dictionary<int, DropGroup> rows;

    internal DropGroupTable(Dictionary<int, DropGroup> rows)
    {
        this.rows = rows;
    }

    internal static DropGroupTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<DropGroup>("DropGroup", DropGroup.Decode));
    }

    internal static DropGroupTable FromRows(List<DropGroup> rows)
    {
        return new DropGroupTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, DropGroup> Rows => rows;
    public DropGroup? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "DropGroup";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "DropGroup";
    public int Count => rows.Count;
}

public sealed class DropEntryTable : ISoraTable
{
    private readonly List<DropEntry> rows;

    internal DropEntryTable(List<DropEntry> rows)
    {
        this.rows = rows;
    }

    internal static DropEntryTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<DropEntry>("DropEntry", DropEntry.Decode));
    }

    internal static DropEntryTable FromRows(List<DropEntry> rows)
    {
        return new DropEntryTable(rows);
    }

    public List<DropEntry> Rows => rows;
    public string Name => "DropEntry";
    public SoraTableMode Mode => SoraTableMode.List;
    public string? Key => null;
    public string RowType => "DropEntry";
    public int Count => rows.Count;
}

public sealed class MonsterTable : ISoraTable
{
    private readonly Dictionary<int, Monster> rows;

    internal MonsterTable(Dictionary<int, Monster> rows)
    {
        this.rows = rows;
    }

    internal static MonsterTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Monster>("Monster", Monster.Decode));
    }

    internal static MonsterTable FromRows(List<Monster> rows)
    {
        return new MonsterTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Monster> Rows => rows;
    public Monster? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Monster";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Monster";
    public int Count => rows.Count;
}

public sealed class StageTable : ISoraTable
{
    private readonly Dictionary<int, Stage> rows;

    internal StageTable(Dictionary<int, Stage> rows)
    {
        this.rows = rows;
    }

    internal static StageTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Stage>("Stage", Stage.Decode));
    }

    internal static StageTable FromRows(List<Stage> rows)
    {
        return new StageTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Stage> Rows => rows;
    public Stage? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Stage";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Stage";
    public int Count => rows.Count;
}

public sealed class StageRewardTable : ISoraTable
{
    private readonly List<StageReward> rows;

    internal StageRewardTable(List<StageReward> rows)
    {
        this.rows = rows;
    }

    internal static StageRewardTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<StageReward>("StageReward", StageReward.Decode));
    }

    internal static StageRewardTable FromRows(List<StageReward> rows)
    {
        return new StageRewardTable(rows);
    }

    public List<StageReward> Rows => rows;
    public string Name => "StageReward";
    public SoraTableMode Mode => SoraTableMode.List;
    public string? Key => null;
    public string RowType => "StageReward";
    public int Count => rows.Count;
}

public sealed class DungeonTable : ISoraTable
{
    private readonly Dictionary<int, Dungeon> rows;

    internal DungeonTable(Dictionary<int, Dungeon> rows)
    {
        this.rows = rows;
    }

    internal static DungeonTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Dungeon>("Dungeon", Dungeon.Decode));
    }

    internal static DungeonTable FromRows(List<Dungeon> rows)
    {
        return new DungeonTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Dungeon> Rows => rows;
    public Dungeon? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Dungeon";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Dungeon";
    public int Count => rows.Count;
}

public sealed class ShopTable : ISoraTable
{
    private readonly Dictionary<int, Shop> rows;

    internal ShopTable(Dictionary<int, Shop> rows)
    {
        this.rows = rows;
    }

    internal static ShopTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Shop>("Shop", Shop.Decode));
    }

    internal static ShopTable FromRows(List<Shop> rows)
    {
        return new ShopTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Shop> Rows => rows;
    public Shop? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Shop";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Shop";
    public int Count => rows.Count;
}

public sealed class ShopItemTable : ISoraTable
{
    private readonly List<ShopItem> rows;

    internal ShopItemTable(List<ShopItem> rows)
    {
        this.rows = rows;
    }

    internal static ShopItemTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<ShopItem>("ShopItem", ShopItem.Decode));
    }

    internal static ShopItemTable FromRows(List<ShopItem> rows)
    {
        return new ShopItemTable(rows);
    }

    public List<ShopItem> Rows => rows;
    public string Name => "ShopItem";
    public SoraTableMode Mode => SoraTableMode.List;
    public string? Key => null;
    public string RowType => "ShopItem";
    public int Count => rows.Count;
}

public sealed class RecipeTable : ISoraTable
{
    private readonly Dictionary<int, Recipe> rows;

    internal RecipeTable(Dictionary<int, Recipe> rows)
    {
        this.rows = rows;
    }

    internal static RecipeTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Recipe>("Recipe", Recipe.Decode));
    }

    internal static RecipeTable FromRows(List<Recipe> rows)
    {
        return new RecipeTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Recipe> Rows => rows;
    public Recipe? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Recipe";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Recipe";
    public int Count => rows.Count;
}

public sealed class GachaPoolTable : ISoraTable
{
    private readonly Dictionary<int, GachaPool> rows;

    internal GachaPoolTable(Dictionary<int, GachaPool> rows)
    {
        this.rows = rows;
    }

    internal static GachaPoolTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<GachaPool>("GachaPool", GachaPool.Decode));
    }

    internal static GachaPoolTable FromRows(List<GachaPool> rows)
    {
        return new GachaPoolTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, GachaPool> Rows => rows;
    public GachaPool? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "GachaPool";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "GachaPool";
    public int Count => rows.Count;
}

public sealed class GachaItemTable : ISoraTable
{
    private readonly List<GachaItem> rows;

    internal GachaItemTable(List<GachaItem> rows)
    {
        this.rows = rows;
    }

    internal static GachaItemTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<GachaItem>("GachaItem", GachaItem.Decode));
    }

    internal static GachaItemTable FromRows(List<GachaItem> rows)
    {
        return new GachaItemTable(rows);
    }

    public List<GachaItem> Rows => rows;
    public string Name => "GachaItem";
    public SoraTableMode Mode => SoraTableMode.List;
    public string? Key => null;
    public string RowType => "GachaItem";
    public int Count => rows.Count;
}

public sealed class EquipmentSetTable : ISoraTable
{
    private readonly Dictionary<int, EquipmentSet> rows;

    internal EquipmentSetTable(Dictionary<int, EquipmentSet> rows)
    {
        this.rows = rows;
    }

    internal static EquipmentSetTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<EquipmentSet>("EquipmentSet", EquipmentSet.Decode));
    }

    internal static EquipmentSetTable FromRows(List<EquipmentSet> rows)
    {
        return new EquipmentSetTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, EquipmentSet> Rows => rows;
    public EquipmentSet? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "EquipmentSet";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "EquipmentSet";
    public int Count => rows.Count;
}

public sealed class AchievementTable : ISoraTable
{
    private readonly Dictionary<int, Achievement> rows;

    internal AchievementTable(Dictionary<int, Achievement> rows)
    {
        this.rows = rows;
    }

    internal static AchievementTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Achievement>("Achievement", Achievement.Decode));
    }

    internal static AchievementTable FromRows(List<Achievement> rows)
    {
        return new AchievementTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Achievement> Rows => rows;
    public Achievement? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Achievement";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Achievement";
    public int Count => rows.Count;
}

public sealed class VipLevelTable : ISoraTable
{
    private readonly Dictionary<int, VipLevel> rows;

    internal VipLevelTable(Dictionary<int, VipLevel> rows)
    {
        this.rows = rows;
    }

    internal static VipLevelTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<VipLevel>("VipLevel", VipLevel.Decode));
    }

    internal static VipLevelTable FromRows(List<VipLevel> rows)
    {
        return new VipLevelTable(SoraConfig.DecodeMapTable(rows, row => row.Level));
    }

    public Dictionary<int, VipLevel> Rows => rows;
    public VipLevel? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "VipLevel";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "level";
    public string RowType => "VipLevel";
    public int Count => rows.Count;
}

public sealed class MailTemplateTable : ISoraTable
{
    private readonly Dictionary<int, MailTemplate> rows;

    internal MailTemplateTable(Dictionary<int, MailTemplate> rows)
    {
        this.rows = rows;
    }

    internal static MailTemplateTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<MailTemplate>("MailTemplate", MailTemplate.Decode));
    }

    internal static MailTemplateTable FromRows(List<MailTemplate> rows)
    {
        return new MailTemplateTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, MailTemplate> Rows => rows;
    public MailTemplate? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "MailTemplate";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "MailTemplate";
    public int Count => rows.Count;
}

public sealed class MailRewardTable : ISoraTable
{
    private readonly List<MailReward> rows;

    internal MailRewardTable(List<MailReward> rows)
    {
        this.rows = rows;
    }

    internal static MailRewardTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<MailReward>("MailReward", MailReward.Decode));
    }

    internal static MailRewardTable FromRows(List<MailReward> rows)
    {
        return new MailRewardTable(rows);
    }

    public List<MailReward> Rows => rows;
    public string Name => "MailReward";
    public SoraTableMode Mode => SoraTableMode.List;
    public string? Key => null;
    public string RowType => "MailReward";
    public int Count => rows.Count;
}

public sealed class DialogueTable : ISoraTable
{
    private readonly Dictionary<int, Dialogue> rows;

    internal DialogueTable(Dictionary<int, Dialogue> rows)
    {
        this.rows = rows;
    }

    internal static DialogueTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<Dialogue>("Dialogue", Dialogue.Decode));
    }

    internal static DialogueTable FromRows(List<Dialogue> rows)
    {
        return new DialogueTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, Dialogue> Rows => rows;
    public Dialogue? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "Dialogue";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "Dialogue";
    public int Count => rows.Count;
}

public sealed class EventRuleTable : ISoraTable
{
    private readonly Dictionary<int, EventRule> rows;

    internal EventRuleTable(Dictionary<int, EventRule> rows)
    {
        this.rows = rows;
    }

    internal static EventRuleTable Decode(SoraBundle bundle)
    {
        return FromRows(bundle.DecodeTable<EventRule>("EventRule", EventRule.Decode));
    }

    internal static EventRuleTable FromRows(List<EventRule> rows)
    {
        return new EventRuleTable(SoraConfig.DecodeMapTable(rows, row => row.Id));
    }

    public Dictionary<int, EventRule> Rows => rows;
    public EventRule? Get(int key)
    {
        return rows.TryGetValue(key, out var row) ? row : default;
    }
    public string Name => "EventRule";
    public SoraTableMode Mode => SoraTableMode.Map;
    public string? Key => "id";
    public string RowType => "EventRule";
    public int Count => rows.Count;
}

public sealed class SoraConfig
{
    private readonly Dictionary<string, ISoraTable> tables;

    private SoraConfig(Dictionary<string, ISoraTable> tables)
    {
        this.tables = tables;
    }

    public static SoraConfig FromBytes(byte[] bytes)
    {
        var bundle = SoraBundle.Parse(bytes);
        var tables = new Dictionary<string, ISoraTable>(28);
        tables["Item"] = ItemTable.Decode(bundle);
        tables["Skill"] = SkillTable.Decode(bundle);
        tables["Quest"] = QuestTable.Decode(bundle);
        tables["QuestReward"] = QuestRewardTable.Decode(bundle);
        tables["GameSettings"] = GameSettingsTable.Decode(bundle);
        tables["Localization"] = LocalizationTable.Decode(bundle);
        tables["LevelExp"] = LevelExpTable.Decode(bundle);
        tables["Character"] = CharacterTable.Decode(bundle);
        tables["CharacterSkill"] = CharacterSkillTable.Decode(bundle);
        tables["Buff"] = BuffTable.Decode(bundle);
        tables["DropGroup"] = DropGroupTable.Decode(bundle);
        tables["DropEntry"] = DropEntryTable.Decode(bundle);
        tables["Monster"] = MonsterTable.Decode(bundle);
        tables["Stage"] = StageTable.Decode(bundle);
        tables["StageReward"] = StageRewardTable.Decode(bundle);
        tables["Dungeon"] = DungeonTable.Decode(bundle);
        tables["Shop"] = ShopTable.Decode(bundle);
        tables["ShopItem"] = ShopItemTable.Decode(bundle);
        tables["Recipe"] = RecipeTable.Decode(bundle);
        tables["GachaPool"] = GachaPoolTable.Decode(bundle);
        tables["GachaItem"] = GachaItemTable.Decode(bundle);
        tables["EquipmentSet"] = EquipmentSetTable.Decode(bundle);
        tables["Achievement"] = AchievementTable.Decode(bundle);
        tables["VipLevel"] = VipLevelTable.Decode(bundle);
        tables["MailTemplate"] = MailTemplateTable.Decode(bundle);
        tables["MailReward"] = MailRewardTable.Decode(bundle);
        tables["Dialogue"] = DialogueTable.Decode(bundle);
        tables["EventRule"] = EventRuleTable.Decode(bundle);
        return new SoraConfig(tables);
    }

    public IEnumerable<ISoraTable> Tables => tables.Values;

    private T Table<T>(string name) where T : class, ISoraTable
    {
        if (tables.TryGetValue(name, out var table) && table is T typed)
        {
            return typed;
        }
        throw new SoraReadException($"generated SoraConfig is missing table `{name}` or has an unexpected table type");
    }
    public ItemTable Item => Table<ItemTable>("Item");
    public SkillTable Skill => Table<SkillTable>("Skill");
    public QuestTable Quest => Table<QuestTable>("Quest");
    public QuestRewardTable QuestReward => Table<QuestRewardTable>("QuestReward");
    public GameSettingsTable GameSettings => Table<GameSettingsTable>("GameSettings");
    public LocalizationTable Localization => Table<LocalizationTable>("Localization");
    public LevelExpTable LevelExp => Table<LevelExpTable>("LevelExp");
    public CharacterTable Character => Table<CharacterTable>("Character");
    public CharacterSkillTable CharacterSkill => Table<CharacterSkillTable>("CharacterSkill");
    public BuffTable Buff => Table<BuffTable>("Buff");
    public DropGroupTable DropGroup => Table<DropGroupTable>("DropGroup");
    public DropEntryTable DropEntry => Table<DropEntryTable>("DropEntry");
    public MonsterTable Monster => Table<MonsterTable>("Monster");
    public StageTable Stage => Table<StageTable>("Stage");
    public StageRewardTable StageReward => Table<StageRewardTable>("StageReward");
    public DungeonTable Dungeon => Table<DungeonTable>("Dungeon");
    public ShopTable Shop => Table<ShopTable>("Shop");
    public ShopItemTable ShopItem => Table<ShopItemTable>("ShopItem");
    public RecipeTable Recipe => Table<RecipeTable>("Recipe");
    public GachaPoolTable GachaPool => Table<GachaPoolTable>("GachaPool");
    public GachaItemTable GachaItem => Table<GachaItemTable>("GachaItem");
    public EquipmentSetTable EquipmentSet => Table<EquipmentSetTable>("EquipmentSet");
    public AchievementTable Achievement => Table<AchievementTable>("Achievement");
    public VipLevelTable VipLevel => Table<VipLevelTable>("VipLevel");
    public MailTemplateTable MailTemplate => Table<MailTemplateTable>("MailTemplate");
    public MailRewardTable MailReward => Table<MailRewardTable>("MailReward");
    public DialogueTable Dialogue => Table<DialogueTable>("Dialogue");
    public EventRuleTable EventRule => Table<EventRuleTable>("EventRule");
    internal static Dictionary<TKey, TValue> DecodeMapTable<TKey, TValue>(List<TValue> rows, Func<TValue, TKey> key)
        where TKey : notnull
    {
        return rows.ToDictionary(key);
    }
    internal static Dictionary<TKey, TValue> DecodeUniqueIndex<TKey, TValue>(List<TValue> rows, Func<TValue, TKey> key)
        where TKey : notnull
    {
        return rows.ToDictionary(key);
    }
    internal static Dictionary<TKey, List<TValue>> DecodeIndex<TKey, TValue>(List<TValue> rows, Func<TValue, TKey> key)
        where TKey : notnull
    {
        var index = new Dictionary<TKey, List<TValue>>();
        foreach (var row in rows)
        {
            var indexKey = key(row);
            if (!index.TryGetValue(indexKey, out var values))
            {
                values = new List<TValue>();
                index[indexKey] = values;
            }
            values.Add(row);
        }
        return index;
    }
    internal static T RequireSingletonTable<T>(List<T> rows, string name)
    {
        if (rows.Count != 1)
        {
            throw new SoraReadException($"expected singleton table `{name}` to contain exactly 1 row, got {rows.Count}");
        }
        return rows[0];
    }
}