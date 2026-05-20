package main

import (
	"fmt"
	"os"

	config "sora-showcase-go/internal/showcase"
)

func main() {
	bytes, err := os.ReadFile("../generated/config.sora")
	must(err)
	cfg, err := config.NewSoraConfigFromBytes(bytes)
	must(err)

	sword, ok := cfg.Item().Get(1001)
	check(ok)
	quest, ok := cfg.Quest().Get(5001)
	check(ok)
	settings := cfg.GameSettings().Rows()

	check(sword.Name == "Iron Sword")
	check(sword.ItemType == config.ItemTypeWeapon)
	check(quest.Title == "First Trial")
	check(quest.QuestType == config.QuestTypeMain)
	check(len(quest.Rewards) == 2)
	check(settings.StartingGold == 100)
	check(cfg.Stage().Len() == 40)
	check(cfg.Monster().Len() == 80)
	check(cfg.Localization().Len() == 80)
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
