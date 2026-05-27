package main

import (
	"fmt"
	"os"

	config "sora-showcase-go/internal/showcase"
)

func main() {
	bytes, err := os.ReadFile("../generated/config.sora")
	must(err)
	bundle, err := config.ParseSoraBundle(bytes)
	must(err)
	cfg, err := config.NewSoraConfigFromSource(bundle)
	must(err)
	localeBytes, err := os.ReadFile("../generated/i18n/zh_cn.sora-i18n")
	must(err)
	localePack, err := config.ParseLocalePack(localeBytes)
	must(err)
	i18n := config.NewSoraI18n()
	must(i18n.Mount(cfg, localePack))
	must(i18n.SetLocale("zh_cn"))

	sword, ok := cfg.Item().Get(1001)
	check(ok)
	swordByName, ok := cfg.Item().GetByName("Iron Sword")
	check(ok)
	quest, ok := cfg.Quest().Get(5001)
	check(ok)
	settings := cfg.GameSettings().Rows()

	check(sword.Name == "Iron Sword")
	check(swordByName.Id == 1001)
	check(sword.ItemType == config.ItemTypeWeapon)
	check(containsItem(cfg.Item().FindByItemType(config.ItemTypeWeapon), sword.Id))
	check(quest.Title == "First Trial")
	check(quest.QuestType == config.QuestTypeMain)
	check(len(quest.Rewards) == 2)
	check(settings.StartingGold == 100)
	check(cfg.Stage().Len() == 40)
	check(cfg.Monster().Len() == 80)
	achievement, ok := cfg.Achievement().Get(14001)
	check(ok)
	check(i18n.Text(achievement.TitleKey) == "中文文本 1")
	formatted, err := i18n.Format(achievement.TitleKey, map[string]any{"count": 100})
	must(err)
	check(formatted == "中文文本 1")
	check(cfg.EventRule().Len() == 20)

	eventRule, ok := cfg.EventRule().Get(17001)
	check(ok)
	condition, ok := eventRule.Condition.(config.EventConditionQuestCompleted)
	check(ok && condition.QuestId == 5002)
	firstAction, ok := eventRule.Actions[0].(config.RewardActionAddItem)
	check(ok && firstAction.ItemId == 1007)

	fmt.Printf(
		"loaded %d items, %d skills, %d quests, %d stages, %d event rules; first quest rewards: %d\n",
		cfg.Item().Len(),
		cfg.Skill().Len(),
		cfg.Quest().Len(),
		cfg.Stage().Len(),
		cfg.EventRule().Len(),
		len(quest.Rewards),
	)
}

func must(err error) {
	if err != nil {
		panic(err)
	}
}

func check(condition bool) {
	if !condition {
		panic("showcase assertion failed")
	}
}

func containsItem(items []config.Item, id int32) bool {
	for _, item := range items {
		if item.Id == id {
			return true
		}
	}
	return false
}
