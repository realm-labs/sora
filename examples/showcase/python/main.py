import os
import sys

# Add the current directory to sys.path so we can import the generated module directly.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from generated import (
    LocalePack,
    SoraConfig,
    SoraI18n,
    ItemType,
    QuestType,
)
from generated.event_condition import EventConditionQuestCompleted
from generated.reward_action import RewardActionAddItem


def main():
    bundle_path = os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "../generated/config.sora"
    )
    with open(bundle_path, "rb") as f:
        bytes_data = f.read()

    config = SoraConfig.from_bytes(bytes_data)
    with open(
        os.path.join(
            os.path.dirname(os.path.abspath(__file__)),
            "../generated/i18n/zh_cn.sora-i18n",
        ),
        "rb",
    ) as f:
        locale_pack = LocalePack.parse(f.read())
    i18n = SoraI18n()
    i18n.mount(config, locale_pack)

    # Tables and values validation
    sword = config.item().get(1001)
    assert sword is not None, "item 1001 not found"
    assert sword.name == "Iron Sword"
    assert sword.item_type == ItemType.WEAPON

    sword_by_name = config.item().get_by_name("Iron Sword")
    assert sword_by_name is not None
    assert sword_by_name.id == 1001

    flame_slash = config.skill().get(101)
    assert flame_slash is not None
    assert flame_slash.name == "Flame Slash"

    quest = config.quest().get(5001)
    assert quest is not None
    assert quest.title == "First Trial"
    assert quest.quest_type == QuestType.MAIN
    assert len(quest.rewards) == 2

    achievement = config.achievement().get(14001)
    assert achievement is not None
    assert i18n.text(achievement.title_key) == "中文文本 1"

    # Check search indices
    weapons = config.item().find_by_item_type(ItemType.WEAPON)
    assert any(item.id == sword.id for item in weapons), "sword id not in weapon list"

    # Settings singleton
    settings = config.game_settings().row()
    assert settings.starting_gold == 100

    # Sizes check
    assert config.quest_reward().len() == 49
    assert config.stage().len() == 40
    assert config.monster().len() == 80
    assert config.event_rule().len() == 20

    # Event rule checking
    event_rule = config.event_rule().get(17001)
    assert event_rule is not None
    assert isinstance(event_rule.condition, EventConditionQuestCompleted)
    assert event_rule.condition.quest_id == 5002

    assert len(event_rule.actions) > 0
    first_action = event_rule.actions[0]
    assert isinstance(first_action, RewardActionAddItem)
    assert first_action.item_id == 1007
    assert first_action.count == 3

    print(
        f"Python showcase successfully verified! Loaded {len(config.item().rows())} items, "
        f"{len(config.skill().rows())} skills, {len(config.quest().rows())} quests, "
        f"{len(config.stage().rows())} stages, {len(config.event_rule().rows())} event rules."
    )


if __name__ == "__main__":
    main()
