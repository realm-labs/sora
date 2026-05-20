mod generated;

use generated::{SoraConfig, item_type::ItemType, quest_type::QuestType};

fn main() {
    let config =
        SoraConfig::from_bytes(include_bytes!("../../generated/config.sora")).expect("bundle");
    let sword = config.item().get(1001).expect("item 1001");
    let flame_slash = config.skill().get(101).expect("skill 101");
    let quest = config.quest().get(5001).expect("quest 5001");
    let settings = config.game_settings();

    assert_eq!(sword.name, "Iron Sword");
    assert!(matches!(sword.item_type, ItemType::Weapon));
    assert_eq!(flame_slash.name, "Flame Slash");
    assert_eq!(quest.title, "First Trial");
    assert!(matches!(quest.quest_type, QuestType::Main));
    assert_eq!(quest.rewards.len(), 2);
    assert_eq!(config.quest_reward().len(), 3);
    assert_eq!(config.quest_reward().iter().count(), 3);
    assert_eq!(settings.starting_gold, 100);

    println!(
        "loaded {} items, {} skills, {} quests; first quest rewards: {}",
        config.item().values().count(),
        config.skill().values().count(),
        config.quest().values().count(),
        quest.rewards.len()
    );
}
