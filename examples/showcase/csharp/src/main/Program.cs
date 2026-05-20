using System;
using System.IO;
using System.Linq;

namespace com.sora.showcase;

internal static class Program
{
    private static void Main()
    {
        var bytes = File.ReadAllBytes(ConfigPath());
        var config = SoraConfig.FromBytes(bytes);
        var sword = config.Item.Get(1001) ?? throw new InvalidOperationException("item 1001");
        var swordByName = config.Item.GetByName("Iron Sword") ?? throw new InvalidOperationException("Iron Sword");
        var quest = config.Quest.Get(5001) ?? throw new InvalidOperationException("quest 5001");
        var settings = config.GameSettings.Rows;

        Check(sword.Name == "Iron Sword");
        Check(swordByName.Id == 1001);
        Check(sword.ItemType == ItemType.Weapon);
        Check(quest.Title == "First Trial");
        Check(quest.QuestType == QuestType.Main);
        Check(quest.Rewards.Count == 2);
        Check(settings.StartingGold == 100);
        Check(config.Stage.Count == 40);
        Check(config.Monster.Count == 80);
        Check(config.Localization.Count == 80);
        Check(config.EventRule.Count == 20);

        var eventRule = config.EventRule.Get(17001) ?? throw new InvalidOperationException("event rule 17001");
        Check(eventRule.Condition is EventCondition.QuestCompleted { QuestId: 5002 });
        Check(eventRule.Actions.First() is RewardAction.AddItem { ItemId: 1007 });

        Console.WriteLine(
            $"loaded {config.Item.Count} items, {config.Skill.Count} skills, {config.Quest.Count} quests, " +
            $"{config.Stage.Count} stages, {config.EventRule.Count} event rules; first quest rewards: {quest.Rewards.Count}");
    }

    private static string ConfigPath()
    {
        return Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "..", "generated", "config.sora"));
    }

    private static void Check(bool condition)
    {
        if (!condition)
        {
            throw new InvalidOperationException("showcase assertion failed");
        }
    }
}
