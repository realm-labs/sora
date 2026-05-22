mod generated;

use generated::{
    SoraConfig, event_condition::EventCondition, item_type::ItemType, quest_type::QuestType,
    reward_action::RewardAction,
};

fn main() {
    let bundle =
        generated::runtime::SoraBundle::parse(include_bytes!("../../generated/config.sora"))
            .expect("bundle");
    let config = SoraConfig::from_source(&bundle).expect("config");
    let sword = config.item().get(1001).expect("item 1001");
    let sword_by_name = config.item().get_by_name("Iron Sword").expect("Iron Sword");
    let flame_slash = config.skill().get(101).expect("skill 101");
    let quest = config.quest().get(5001).expect("quest 5001");
    let settings = config.game_settings();

    assert_eq!(sword.name.as_ref(), "Iron Sword");
    assert_eq!(sword_by_name.id, 1001);
    assert!(matches!(sword.item_type, ItemType::Weapon));
    assert!(
        config
            .item()
            .find_by_item_type(ItemType::Weapon)
            .any(|item| item.id == sword.id)
    );
    assert_eq!(flame_slash.name.as_ref(), "Flame Slash");
    assert_eq!(quest.title.as_ref(), "First Trial");
    assert!(matches!(quest.quest_type, QuestType::Main));
    assert_eq!(quest.rewards.len(), 2);
    assert_eq!(config.quest_reward().len(), 49);
    assert_eq!(config.quest_reward().iter().count(), 49);
    assert_eq!(config.stage().len(), 40);
    assert_eq!(config.monster().len(), 80);
    assert_eq!(config.localization().len(), 80);
    assert_eq!(config.event_rule().len(), 20);

    let event_rule = config.event_rule().get(17001).expect("event rule 17001");
    assert!(matches!(
        &event_rule.condition,
        EventCondition::QuestCompleted { quest_id: 5002 }
    ));
    assert!(matches!(
        &event_rule.actions[0],
        RewardAction::AddItem {
            item_id: 1007,
            count: 3
        }
    ));
    assert_eq!(settings.starting_gold, 100);

    println!(
        "loaded {} items, {} skills, {} quests, {} stages, {} event rules; first quest rewards: {}",
        config.item().values().count(),
        config.skill().values().count(),
        config.quest().values().count(),
        config.stage().values().count(),
        config.event_rule().values().count(),
        quest.rewards.len()
    );
}
