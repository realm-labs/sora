mod generated;

use generated::{SoraConfig, item_type::ItemType, quest_type::QuestType};

fn main() {
    let config =
        SoraConfig::from_bytes(include_bytes!("../../generated/config.sora")).expect("bundle");
    let sword = config.get_item(1001).expect("item 1001");
    let flame_slash = config.get_skill(101).expect("skill 101");
    let quest = config.get_quest(5001).expect("quest 5001");
    let settings = config.game_settings_row();

    assert_eq!(sword.name, "Iron Sword");
    assert!(matches!(sword.item_type, ItemType::Weapon));
    assert_eq!(flame_slash.name, "Flame Slash");
    assert_eq!(quest.title, "First Trial");
    assert!(matches!(quest.quest_type, QuestType::Main));
    assert_eq!(quest.rewards.len(), 2);
    assert_eq!(config.quest_reward_rows().len(), 3);
    assert_eq!(config.iter_quest_reward().count(), 3);
    assert_eq!(settings.starting_gold, 100);

    println!(
        "loaded {} items, {} skills, {} quests; first quest rewards: {}",
        config.iter_item().count(),
        config.iter_skill().count(),
        config.iter_quest().count(),
        quest.rewards.len()
    );
}
