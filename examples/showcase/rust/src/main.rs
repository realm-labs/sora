mod generated;

use generated::{
    SoraConfig, SoraI18n, achievement::AchievementId, complex_action_group::ComplexActionGroupId,
    complex_condition_group::ComplexConditionGroupId, complex_rule::ComplexRuleId,
    event_condition::EventCondition, event_rule::EventRuleId, item::ItemId, item_type::ItemType,
    quest::QuestId, quest_type::QuestType, reward_action::RewardAction, skill::SkillId,
};

fn main() {
    let bundle =
        generated::runtime::SoraBundle::parse(include_bytes!("../../generated/config.sora"))
            .expect("bundle");
    let config = SoraConfig::from_source(&bundle).expect("config");
    let locale_pack = generated::runtime::LocalePack::from_bytes(include_bytes!(
        "../../generated/i18n/zh_cn.sora-i18n"
    ))
    .expect("locale pack");
    let mut i18n = SoraI18n::new();
    i18n.mount(&config, locale_pack).expect("mount zh_cn");
    i18n.set_locale("zh_cn").expect("set zh_cn");
    let sword = config
        .item()
        .get(&ItemId::from_raw(1001))
        .expect("item 1001");
    let sword_by_name = config.item().get_by_name("Iron Sword").expect("Iron Sword");
    let flame_slash = config
        .skill()
        .get(&SkillId::from_raw(101))
        .expect("skill 101");
    let quest = config
        .quest()
        .get(&QuestId::from_raw(5001))
        .expect("quest 5001");
    let settings = config.game_settings();

    assert_eq!(sword.name.as_ref(), "Iron Sword");
    assert_eq!(sword_by_name.id.raw(), 1001);
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
    let achievement = config
        .achievement()
        .get(&AchievementId::from_raw(14001))
        .expect("achievement 14001");
    assert_eq!(i18n.text(&achievement.title_key), "中文文本 1");
    assert_eq!(
        i18n.format(&achievement.title_key, [("count", 100)])
            .expect("formatted title text"),
        "中文文本 1"
    );
    assert_eq!(config.event_rule().len(), 20);

    let event_rule = config
        .event_rule()
        .get(&EventRuleId::from_raw(17001))
        .expect("event rule 17001");
    assert!(matches!(
        &event_rule.condition,
        EventCondition::QuestCompleted { quest_id } if quest_id.raw() == 5002
    ));
    assert!(matches!(
        &event_rule.actions[0],
        RewardAction::AddItem {
            item_id,
            count: 3
        } if item_id.raw() == 1007
    ));
    assert_eq!(settings.starting_gold, 100);
    assert!((settings.gravity - 9.80665).abs() < f64::EPSILON);
    assert_eq!(
        settings
            .daily_bonus_items
            .iter()
            .map(|id| id.raw())
            .collect::<Vec<_>>(),
        [1001, 1002, 2001]
    );
    assert_eq!(settings.spawn_points[1].x, 12.0);
    let maintenance = settings.maintenance.as_ref().expect("maintenance window");
    assert_eq!(maintenance.duration_minutes, 90);
    assert_eq!(config.maintenance_window().len(), 1);

    let complex_rule = config
        .complex_rule()
        .get(&ComplexRuleId::from_raw(18001))
        .expect("complex rule 18001");
    assert!(matches!(
        &complex_rule.root_condition,
        EventCondition::AllConditions {
            condition_group_id
        } if *condition_group_id == ComplexConditionGroupId::from_raw(19001)
    ));
    assert!(matches!(
        &complex_rule.actions[2],
        RewardAction::RunActionGroup {
            action_group_id
        } if *action_group_id == ComplexActionGroupId::from_raw(18103)
    ));
    assert_eq!(complex_rule.budget.random.len(), 2);
    assert_eq!(complex_rule.budget.limits.get("daily").copied(), Some(3));
    let nested_conditions = config
        .complex_condition_group()
        .get(&ComplexConditionGroupId::from_raw(19001))
        .expect("complex condition group 19001");
    assert_eq!(nested_conditions.conditions.len(), 2);

    println!(
        "loaded {} items, {} skills, {} quests, {} stages, {} event rules, {} complex rules; first quest rewards: {}",
        config.item().values().count(),
        config.skill().values().count(),
        config.quest().values().count(),
        config.stage().values().count(),
        config.event_rule().values().count(),
        config.complex_rule().values().count(),
        quest.rewards.len()
    );
}
