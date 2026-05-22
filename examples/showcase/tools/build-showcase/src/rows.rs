pub(crate) fn showcase_rows(table: &str) -> Vec<Vec<String>> {
    match table {
        "Item" => vec![
            vec![
                "1001".into(),
                "Iron Sword".into(),
                "Weapon".into(),
                "1".into(),
                "Gold,0,120".into(),
                "[\"starter\",\"melee\"]".into(),
                "tier,1|power,10".into(),
            ],
            vec![
                "1002".into(),
                "Magic Crystal".into(),
                "Material".into(),
                "999".into(),
                "Diamond,0,3".into(),
                "[\"craft\",\"rare\"]".into(),
                "tier,2|power,11".into(),
            ],
            vec![
                "2001".into(),
                "Health Potion".into(),
                "Consumable".into(),
                "50".into(),
                "Gold,0,25".into(),
                "[\"potion\",\"recover\"]".into(),
                "tier,3|power,12".into(),
            ],
            vec![
                "3001".into(),
                "Training Medal".into(),
                "Currency".into(),
                "".into(),
                "Gold,0,1".into(),
                "[\"quest\",\"token\"]".into(),
                "tier,4|power,13".into(),
            ],
        ]
        .into_iter()
        .chain((1003..=1120).map(|id| {
            let item_type = match id % 5 {
                0 => "Weapon",
                1 => "Armor",
                2 => "Currency",
                3 => "Material",
                _ => "Consumable",
            };
            vec![
                id.to_string(),
                format!("Item {id}"),
                item_type.to_owned(),
                if item_type == "Weapon" || item_type == "Armor" {
                    "1".to_owned()
                } else {
                    ((id % 99) + 1).to_string()
                },
                format!("Gold,0,{}", 10 + id % 500),
                format!("[\"auto\",\"{}\"]", item_type.to_ascii_lowercase()),
                format!("tier,{}|power,{}", ((id - 999) % 5) + 1, id - 989),
            ]
        }))
        .collect(),
        "Skill" => vec![
            vec![
                "101".into(),
                "Flame Slash".into(),
                "Fire".into(),
                "Gold,0,150".into(),
                "{\"element\":\"Fire\",\"power\":120,\"radius\":2.5}".into(),
                "3".into(),
                "1001".into(),
                "0,1.2,0".into(),
            ],
            vec![
                "102".into(),
                "Ice Lance".into(),
                "Ice".into(),
                "Item,1002,2".into(),
                "{\"element\":\"Ice\",\"power\":95}".into(),
                "".into(),
                "".into(),
                "0,1.5,3".into(),
            ],
        ]
        .into_iter()
        .chain((103..=130).map(|id| {
            let element = element(id);
            vec![
                id.to_string(),
                format!("{element} Skill {id}"),
                element.to_owned(),
                format!("Gold,0,{}", id * 3),
                format!(
                    "{{\"element\":\"{element}\",\"power\":{},\"radius\":{}}}",
                    80 + id % 240,
                    1.0 + (id % 5) as f32 * 0.5
                ),
                ((id % 50) + 1).to_string(),
                if id % 3 == 0 {
                    "1002".to_owned()
                } else {
                    String::new()
                },
                format!("{},{},{}", id % 5, 1.0 + (id % 4) as f32, id % 7),
            ]
        }))
        .collect(),
        "Quest" => vec![
            vec![
                "5001".into(),
                "Main".into(),
                "First Trial".into(),
                "1001".into(),
                "[101,102]".into(),
                "12,0,5".into(),
                "".into(),
            ],
            vec![
                "5002".into(),
                "Daily".into(),
                "Crystal Supply".into(),
                "1002".into(),
                "[102]".into(),
                "2,0,8".into(),
                "".into(),
            ],
        ]
        .into_iter()
        .chain((5003..=5025).map(|id| {
            let kind = match id % 3 {
                0 => "Main",
                1 => "Side",
                _ => "Daily",
            };
            vec![
                id.to_string(),
                kind.to_owned(),
                format!("Quest {id}"),
                item_id(id).to_string(),
                format!("[{},{}]", skill_id(id), skill_id(id + 1)),
                format!("{},{},{}", id % 50, 0, id % 30),
                String::new(),
            ]
        }))
        .collect(),
        "QuestReward" => quest_reward_rows(),
        "GameSettings" => vec![vec![
            "2026.05".into(),
            "5".into(),
            "".into(),
            "0,0,0".into(),
            "[1001,2001]".into(),
        ]],
        "Localization" => localization_rows(),
        "LevelExp" => (1..=100)
            .map(|level| {
                vec![
                    level.to_string(),
                    (level as i64 * level as i64 * 100).to_string(),
                    if level % 10 == 0 {
                        format!("feature_{level}")
                    } else {
                        String::new()
                    },
                ]
            })
            .collect(),
        "Character" => (4001..=4020)
            .map(|id| {
                vec![
                    id.to_string(),
                    format!("Hero {id}"),
                    rarity(id).to_owned(),
                    ((id % 80) + 1).to_string(),
                    skill_id(id).to_string(),
                    format!("[1001,{},{}]", item_id(id), item_id(id + 1)),
                    format!("{},{},{}", id % 8, 0, id % 6),
                ]
            })
            .collect(),
        "CharacterSkill" => (4001..=4020)
            .flat_map(|character_id| {
                (0..3).map(move |offset| {
                    vec![
                        character_id.to_string(),
                        skill_id(character_id + offset).to_string(),
                        (1 + offset * 10).to_string(),
                    ]
                })
            })
            .collect(),
        "Buff" => (6001..=6020)
            .map(|id| {
                vec![
                    id.to_string(),
                    format!("Buff {id}"),
                    format!("{}", 3.0 + (id % 8) as f32),
                    format!(
                        "[{{\"stat\":\"{}\",\"value\":{},\"is_percent\":{}}}]",
                        stat(id),
                        5.0 + (id % 20) as f32,
                        if id % 2 == 0 { "true" } else { "false" }
                    ),
                ]
            })
            .collect(),
        "DropGroup" => (7001..=7020)
            .map(|id| vec![id.to_string(), format!("Drop Group {id}")])
            .collect(),
        "DropEntry" => (7001..=7020)
            .flat_map(|group_id| {
                (1..=3).map(move |seq| {
                    vec![
                        group_id.to_string(),
                        seq.to_string(),
                        item_id(group_id + seq).to_string(),
                        (1 + seq * 2).to_string(),
                        format!("{}", 10.0 + seq as f32 * 5.0),
                    ]
                })
            })
            .collect(),
        "Monster" => (8001..=8080)
            .map(|id| {
                vec![
                    id.to_string(),
                    format!("Monster {id}"),
                    ((id % 80) + 1).to_string(),
                    element(id).to_owned(),
                    (7001 + id % 20).to_string(),
                    format!("{},{},{}", id % 20, 0, id % 15),
                ]
            })
            .collect(),
        "Stage" => (9001..=9040)
            .map(|id| {
                vec![
                    id.to_string(),
                    format!("Stage {id}"),
                    format!(
                        "[{},{},{}]",
                        monster_id(id),
                        monster_id(id + 1),
                        monster_id(id + 2)
                    ),
                    (id * 12).to_string(),
                    String::new(),
                ]
            })
            .collect(),
        "StageReward" => (9001..=9040)
            .flat_map(|stage_id| {
                (1..=2).map(move |seq| {
                    vec![
                        stage_id.to_string(),
                        seq.to_string(),
                        item_id(stage_id + seq).to_string(),
                        (seq * 3).to_string(),
                    ]
                })
            })
            .collect(),
        "Dungeon" => (1..=10)
            .map(|index| {
                let start = 9001 + (index - 1) * 4;
                vec![
                    (9500 + index).to_string(),
                    format!("Dungeon {index}"),
                    format!("[{start},{},{},{}]", start + 1, start + 2, start + 3),
                    format!("Gold,0,{}", index * 100),
                ]
            })
            .collect(),
        "Shop" => (1..=5)
            .map(|index| {
                vec![
                    (10000 + index).to_string(),
                    format!("Shop {index}"),
                    if index % 2 == 0 { "Diamond" } else { "Gold" }.to_owned(),
                ]
            })
            .collect(),
        "ShopItem" => (1..=5)
            .flat_map(|shop_index| {
                (1..=10).map(move |seq| {
                    vec![
                        (10000 + shop_index).to_string(),
                        seq.to_string(),
                        item_id(shop_index * 10 + seq).to_string(),
                        format!("Gold,0,{}", 20 + seq * 7),
                        if seq % 3 == 0 {
                            "5".to_owned()
                        } else {
                            String::new()
                        },
                    ]
                })
            })
            .collect(),
        "Recipe" => (1..=30)
            .map(|index| {
                vec![
                    (11000 + index).to_string(),
                    item_id(index).to_string(),
                    format!(
                        "Item,{},{}|Gold,0,{}",
                        item_id(index + 1),
                        2 + index % 4,
                        100 + index * 10
                    ),
                ]
            })
            .collect(),
        "GachaPool" => (1..=3)
            .map(|index| {
                vec![
                    (12000 + index).to_string(),
                    format!("Pool {index}"),
                    format!("Diamond,0,{}", index * 10),
                ]
            })
            .collect(),
        "GachaItem" => (1..=3)
            .flat_map(|pool_index| {
                (1..=20).map(move |seq| {
                    vec![
                        (12000 + pool_index).to_string(),
                        item_id(pool_index * 30 + seq).to_string(),
                        rarity(seq).to_owned(),
                        format!("{}", 1.0 + (seq % 10) as f32),
                    ]
                })
            })
            .collect(),
        "EquipmentSet" => (1..=10)
            .map(|index| {
                vec![
                    (13000 + index).to_string(),
                    format!("Set {index}"),
                    format!(
                        "[{},{},{}]",
                        item_id(index),
                        item_id(index + 1),
                        item_id(index + 2)
                    ),
                    format!(
                        "{{\"element\":\"{}\",\"power\":{},\"radius\":1.0}}",
                        element(index),
                        50 + index
                    ),
                ]
            })
            .collect(),
        "Achievement" => (1..=30)
            .map(|index| {
                vec![
                    (14000 + index).to_string(),
                    loc_key(index),
                    (index as i64 * 1000).to_string(),
                    format!("Gold,0,{}", index * 50),
                ]
            })
            .collect(),
        "VipLevel" => (1..=10)
            .map(|level| {
                vec![
                    level.to_string(),
                    format!("Diamond,0,{}", level * 100),
                    format!("[\"fast_battle\",\"shop_discount_{}\"]", level),
                ]
            })
            .collect(),
        "MailTemplate" => (1..=20)
            .map(|index| {
                vec![
                    (15000 + index).to_string(),
                    match index % 3 {
                        0 => "System",
                        1 => "Event",
                        _ => "Compensation",
                    }
                    .to_owned(),
                    loc_key(index),
                    loc_key(index + 20),
                    String::new(),
                ]
            })
            .collect(),
        "MailReward" => (1..=20)
            .flat_map(|index| {
                (1..=2).map(move |seq| {
                    vec![
                        (15000 + index).to_string(),
                        seq.to_string(),
                        item_id(index + seq).to_string(),
                        (seq * 2).to_string(),
                    ]
                })
            })
            .collect(),
        "Dialogue" => (1..=20)
            .map(|index| {
                vec![
                    (16000 + index).to_string(),
                    loc_key(index + 40),
                    format!(
                        "[\"dialogue line {}-1\",\"dialogue line {}-2\"]",
                        index, index
                    ),
                ]
            })
            .collect(),
        "EventRule" => (1..=20)
            .map(|index| {
                let condition = match index % 3 {
                    0 => format!("{{\"type\":\"LevelAtLeast\",\"level\":{}}}", 1 + index),
                    1 => format!("{{\"type\":\"QuestCompleted\",\"quest_id\":{}}}", 5001 + index % 25),
                    _ => format!(
                        "{{\"type\":\"HasItem\",\"item_id\":{},\"count\":{}}}",
                        item_id(index),
                        1 + index % 5
                    ),
                };
                let actions = format!(
                    "[{{\"type\":\"AddItem\",\"item_id\":{},\"count\":{} }},{{\"type\":\"AddBuff\",\"buff_id\":{},\"duration\":{} }},{{\"type\":\"UnlockStage\",\"stage_id\":{} }}]",
                    item_id(index + 3),
                    2 + index % 4,
                    6001 + index % 20,
                    5.0 + (index % 5) as f32,
                    9001 + index % 40
                );
                vec![
                    (17000 + index).to_string(),
                    format!("Event Rule {index}"),
                    condition,
                    actions,
                ]
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn localization_rows() -> Vec<Vec<String>> {
    (1..=80)
        .map(|index| {
            let key = loc_key(index);
            vec![
                key.clone(),
                format!("中文文本 {index}"),
                format!("English Text {index}"),
                if index % 10 == 0 {
                    format!("note {index}")
                } else {
                    String::new()
                },
            ]
        })
        .collect()
}

fn quest_reward_rows() -> Vec<Vec<String>> {
    let mut rows = vec![
        vec!["5001".into(), "1".into(), "3001".into(), "5".into()],
        vec!["5001".into(), "2".into(), "2001".into(), "3".into()],
        vec!["5002".into(), "1".into(), "1002".into(), "1".into()],
    ];
    rows.extend((5003..=5025).flat_map(|quest_id| {
        (1..=2).map(move |seq| {
            vec![
                quest_id.to_string(),
                seq.to_string(),
                item_id(quest_id + seq).to_string(),
                (seq * 2).to_string(),
            ]
        })
    }));
    rows
}

fn item_id(seed: i32) -> i32 {
    1003 + seed.rem_euclid(118)
}

fn skill_id(seed: i32) -> i32 {
    101 + seed.rem_euclid(30)
}

fn monster_id(seed: i32) -> i32 {
    8001 + seed.rem_euclid(80)
}

fn loc_key(index: i32) -> String {
    format!("loc_{index:03}")
}

fn element(seed: i32) -> &'static str {
    match seed.rem_euclid(4) {
        0 => "Fire",
        1 => "Ice",
        2 => "Lightning",
        _ => "Physical",
    }
}

fn rarity(seed: i32) -> &'static str {
    match seed.rem_euclid(5) {
        0 => "Common",
        1 => "Uncommon",
        2 => "Rare",
        3 => "Epic",
        _ => "Legendary",
    }
}

fn stat(seed: i32) -> &'static str {
    match seed.rem_euclid(5) {
        0 => "Hp",
        1 => "Attack",
        2 => "Defense",
        3 => "Speed",
        _ => "CritRate",
    }
}
