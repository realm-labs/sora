import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { ItemType } from "./generated/item_type.js";
import { SoraConfig } from "./generated/sora_config.js";
import { SoraBundle } from "./generated/sora_runtime.js";

const root = dirname(fileURLToPath(import.meta.url));
const bytes = readFileSync(join(root, "../generated/config.sora"));
const config = SoraConfig.fromSource(SoraBundle.parse(bytes));

const sword = config.item().get(1001);
const swordByName = config.item().getByName("Iron Sword");
const quest = config.quest().get(5001);
const settings = config.gameSettings().row();
const eventRule = config.eventRule().get(17001);

check(sword?.name === "Iron Sword");
check(swordByName?.id === 1001);
check(sword?.itemType === ItemType.Weapon);
check(config.item().findByItemType(ItemType.Weapon).some((item) => item.id === sword.id));
check(quest?.title === "First Trial");
check(quest?.questType === "Main");
check(quest?.rewards.length === 2);
check(settings.startingGold === 100);
check(config.stage().len() === 40);
check(config.monster().len() === 80);
check(config.localization().len() === 80);
check(config.eventRule().len() === 20);
check(eventRule?.condition.type === "QuestCompleted");
check(eventRule.condition.questId === 5002);
check(eventRule.actions[0]?.type === "AddItem");
check(eventRule.actions[0].itemId === 1007);

console.log(
    `loaded ${config.item().len()} items, ${config.skill().len()} skills, ${config.quest().len()} quests, ${config.eventRule().len()} event rules`,
);

function check(condition) {
    if (!condition) {
        throw new Error("showcase assertion failed");
    }
}
