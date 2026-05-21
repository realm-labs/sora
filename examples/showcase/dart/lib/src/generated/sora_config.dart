import 'runtime.dart';
import 'item_type.dart';
import 'resource_kind.dart';
import 'element_type.dart';
import 'quest_type.dart';
import 'rarity.dart';
import 'stat_type.dart';
import 'mail_type.dart';
import 'resource_cost.dart';
import 'vec3.dart';
import 'skill_effect.dart';
import 'reward.dart';
import 'stat_modifier.dart';
import 'item.dart';
import 'skill.dart';
import 'quest.dart';
import 'quest_reward.dart';
import 'game_settings.dart';
import 'localization.dart';
import 'level_exp.dart';
import 'character.dart';
import 'character_skill.dart';
import 'buff.dart';
import 'drop_group.dart';
import 'drop_entry.dart';
import 'monster.dart';
import 'stage.dart';
import 'stage_reward.dart';
import 'dungeon.dart';
import 'shop.dart';
import 'shop_item.dart';
import 'recipe.dart';
import 'gacha_pool.dart';
import 'gacha_item.dart';
import 'equipment_set.dart';
import 'achievement.dart';
import 'vip_level.dart';
import 'mail_template.dart';
import 'mail_reward.dart';
import 'dialogue.dart';
import 'event_rule.dart';
import 'event_condition.dart';
import 'reward_action.dart';

abstract interface class SoraConfigTable {
  String get name;
  String get mode;
  String? get key;
  int get length;
}

final class ItemTable extends Iterable<Item> implements SoraConfigTable {
  final Map<int, Item> _rows;
  final Map<String, Item> _name;
  final Map<ItemType, List<Item>> _itemType;

  const ItemTable(
    this._rows,
    this._name,
    this._itemType,
  );

  static ItemTable decode(List<Item> rows) {
    return ItemTable(
      decodeMapTable(rows, (row) => row.id),
      decodeUniqueIndex(rows, (row) => row.name),
      decodeIndex(rows, (row) => row.itemType),
    );
  }

  @override
  String get name => 'Item';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Item> get iterator => _rows.values.iterator;
  Item? operator [](int key) => _rows[key];

  Item get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Item` for key `$key`');
    }
    return row;
  }

  Map<int, Item> get rows => _rows;
  Item? getByName(String name) =>
      _name[name];
  List<Item> findByItemType(ItemType itemType) =>
      _itemType[itemType] ?? const [];
}

final class SkillTable extends Iterable<Skill> implements SoraConfigTable {
  final Map<int, Skill> _rows;

  const SkillTable(
    this._rows,
  );

  static SkillTable decode(List<Skill> rows) {
    return SkillTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Skill';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Skill> get iterator => _rows.values.iterator;
  Skill? operator [](int key) => _rows[key];

  Skill get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Skill` for key `$key`');
    }
    return row;
  }

  Map<int, Skill> get rows => _rows;
}

final class QuestTable extends Iterable<Quest> implements SoraConfigTable {
  final Map<int, Quest> _rows;

  const QuestTable(
    this._rows,
  );

  static QuestTable decode(List<Quest> rows) {
    return QuestTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Quest';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Quest> get iterator => _rows.values.iterator;
  Quest? operator [](int key) => _rows[key];

  Quest get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Quest` for key `$key`');
    }
    return row;
  }

  Map<int, Quest> get rows => _rows;
}

final class QuestRewardTable extends Iterable<QuestReward> implements SoraConfigTable {
  final List<QuestReward> _rows;

  const QuestRewardTable(
    this._rows,
  );

  static QuestRewardTable decode(List<QuestReward> rows) {
    return QuestRewardTable(
      rows,
    );
  }

  @override
  String get name => 'QuestReward';

  @override
  String get mode => 'list';

  @override
  String? get key => null;

  @override
  int get length => _rows.length;

  @override
  Iterator<QuestReward> get iterator => _rows.iterator;
  List<QuestReward> get rows => _rows;
}

final class GameSettingsTable extends Iterable<GameSettings> implements SoraConfigTable {
  final GameSettings _row;

  const GameSettingsTable(
    this._row,
  );

  static GameSettingsTable decode(List<GameSettings> rows) {
    return GameSettingsTable(
      requireSingletonTable(rows, 'GameSettings'),
    );
  }

  @override
  String get name => 'GameSettings';

  @override
  String get mode => 'singleton';

  @override
  String? get key => null;

  @override
  int get length => 1;

  @override
  Iterator<GameSettings> get iterator => <GameSettings>[_row].iterator;
  GameSettings get row => _row;
}

final class LocalizationTable extends Iterable<Localization> implements SoraConfigTable {
  final Map<String, Localization> _rows;

  const LocalizationTable(
    this._rows,
  );

  static LocalizationTable decode(List<Localization> rows) {
    return LocalizationTable(
      decodeMapTable(rows, (row) => row.key),
    );
  }

  @override
  String get name => 'Localization';

  @override
  String get mode => 'map';

  @override
  String? get key => 'key';

  @override
  int get length => _rows.length;

  @override
  Iterator<Localization> get iterator => _rows.values.iterator;
  Localization? operator [](String key) => _rows[key];

  Localization get(String key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Localization` for key `$key`');
    }
    return row;
  }

  Map<String, Localization> get rows => _rows;
}

final class LevelExpTable extends Iterable<LevelExp> implements SoraConfigTable {
  final Map<int, LevelExp> _rows;

  const LevelExpTable(
    this._rows,
  );

  static LevelExpTable decode(List<LevelExp> rows) {
    return LevelExpTable(
      decodeMapTable(rows, (row) => row.level),
    );
  }

  @override
  String get name => 'LevelExp';

  @override
  String get mode => 'map';

  @override
  String? get key => 'level';

  @override
  int get length => _rows.length;

  @override
  Iterator<LevelExp> get iterator => _rows.values.iterator;
  LevelExp? operator [](int key) => _rows[key];

  LevelExp get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `LevelExp` for key `$key`');
    }
    return row;
  }

  Map<int, LevelExp> get rows => _rows;
}

final class CharacterTable extends Iterable<Character> implements SoraConfigTable {
  final Map<int, Character> _rows;

  const CharacterTable(
    this._rows,
  );

  static CharacterTable decode(List<Character> rows) {
    return CharacterTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Character';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Character> get iterator => _rows.values.iterator;
  Character? operator [](int key) => _rows[key];

  Character get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Character` for key `$key`');
    }
    return row;
  }

  Map<int, Character> get rows => _rows;
}

final class CharacterSkillTable extends Iterable<CharacterSkill> implements SoraConfigTable {
  final List<CharacterSkill> _rows;

  const CharacterSkillTable(
    this._rows,
  );

  static CharacterSkillTable decode(List<CharacterSkill> rows) {
    return CharacterSkillTable(
      rows,
    );
  }

  @override
  String get name => 'CharacterSkill';

  @override
  String get mode => 'list';

  @override
  String? get key => null;

  @override
  int get length => _rows.length;

  @override
  Iterator<CharacterSkill> get iterator => _rows.iterator;
  List<CharacterSkill> get rows => _rows;
}

final class BuffTable extends Iterable<Buff> implements SoraConfigTable {
  final Map<int, Buff> _rows;

  const BuffTable(
    this._rows,
  );

  static BuffTable decode(List<Buff> rows) {
    return BuffTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Buff';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Buff> get iterator => _rows.values.iterator;
  Buff? operator [](int key) => _rows[key];

  Buff get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Buff` for key `$key`');
    }
    return row;
  }

  Map<int, Buff> get rows => _rows;
}

final class DropGroupTable extends Iterable<DropGroup> implements SoraConfigTable {
  final Map<int, DropGroup> _rows;

  const DropGroupTable(
    this._rows,
  );

  static DropGroupTable decode(List<DropGroup> rows) {
    return DropGroupTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'DropGroup';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<DropGroup> get iterator => _rows.values.iterator;
  DropGroup? operator [](int key) => _rows[key];

  DropGroup get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `DropGroup` for key `$key`');
    }
    return row;
  }

  Map<int, DropGroup> get rows => _rows;
}

final class DropEntryTable extends Iterable<DropEntry> implements SoraConfigTable {
  final List<DropEntry> _rows;

  const DropEntryTable(
    this._rows,
  );

  static DropEntryTable decode(List<DropEntry> rows) {
    return DropEntryTable(
      rows,
    );
  }

  @override
  String get name => 'DropEntry';

  @override
  String get mode => 'list';

  @override
  String? get key => null;

  @override
  int get length => _rows.length;

  @override
  Iterator<DropEntry> get iterator => _rows.iterator;
  List<DropEntry> get rows => _rows;
}

final class MonsterTable extends Iterable<Monster> implements SoraConfigTable {
  final Map<int, Monster> _rows;

  const MonsterTable(
    this._rows,
  );

  static MonsterTable decode(List<Monster> rows) {
    return MonsterTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Monster';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Monster> get iterator => _rows.values.iterator;
  Monster? operator [](int key) => _rows[key];

  Monster get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Monster` for key `$key`');
    }
    return row;
  }

  Map<int, Monster> get rows => _rows;
}

final class StageTable extends Iterable<Stage> implements SoraConfigTable {
  final Map<int, Stage> _rows;

  const StageTable(
    this._rows,
  );

  static StageTable decode(List<Stage> rows) {
    return StageTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Stage';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Stage> get iterator => _rows.values.iterator;
  Stage? operator [](int key) => _rows[key];

  Stage get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Stage` for key `$key`');
    }
    return row;
  }

  Map<int, Stage> get rows => _rows;
}

final class StageRewardTable extends Iterable<StageReward> implements SoraConfigTable {
  final List<StageReward> _rows;

  const StageRewardTable(
    this._rows,
  );

  static StageRewardTable decode(List<StageReward> rows) {
    return StageRewardTable(
      rows,
    );
  }

  @override
  String get name => 'StageReward';

  @override
  String get mode => 'list';

  @override
  String? get key => null;

  @override
  int get length => _rows.length;

  @override
  Iterator<StageReward> get iterator => _rows.iterator;
  List<StageReward> get rows => _rows;
}

final class DungeonTable extends Iterable<Dungeon> implements SoraConfigTable {
  final Map<int, Dungeon> _rows;

  const DungeonTable(
    this._rows,
  );

  static DungeonTable decode(List<Dungeon> rows) {
    return DungeonTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Dungeon';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Dungeon> get iterator => _rows.values.iterator;
  Dungeon? operator [](int key) => _rows[key];

  Dungeon get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Dungeon` for key `$key`');
    }
    return row;
  }

  Map<int, Dungeon> get rows => _rows;
}

final class ShopTable extends Iterable<Shop> implements SoraConfigTable {
  final Map<int, Shop> _rows;

  const ShopTable(
    this._rows,
  );

  static ShopTable decode(List<Shop> rows) {
    return ShopTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Shop';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Shop> get iterator => _rows.values.iterator;
  Shop? operator [](int key) => _rows[key];

  Shop get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Shop` for key `$key`');
    }
    return row;
  }

  Map<int, Shop> get rows => _rows;
}

final class ShopItemTable extends Iterable<ShopItem> implements SoraConfigTable {
  final List<ShopItem> _rows;

  const ShopItemTable(
    this._rows,
  );

  static ShopItemTable decode(List<ShopItem> rows) {
    return ShopItemTable(
      rows,
    );
  }

  @override
  String get name => 'ShopItem';

  @override
  String get mode => 'list';

  @override
  String? get key => null;

  @override
  int get length => _rows.length;

  @override
  Iterator<ShopItem> get iterator => _rows.iterator;
  List<ShopItem> get rows => _rows;
}

final class RecipeTable extends Iterable<Recipe> implements SoraConfigTable {
  final Map<int, Recipe> _rows;

  const RecipeTable(
    this._rows,
  );

  static RecipeTable decode(List<Recipe> rows) {
    return RecipeTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Recipe';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Recipe> get iterator => _rows.values.iterator;
  Recipe? operator [](int key) => _rows[key];

  Recipe get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Recipe` for key `$key`');
    }
    return row;
  }

  Map<int, Recipe> get rows => _rows;
}

final class GachaPoolTable extends Iterable<GachaPool> implements SoraConfigTable {
  final Map<int, GachaPool> _rows;

  const GachaPoolTable(
    this._rows,
  );

  static GachaPoolTable decode(List<GachaPool> rows) {
    return GachaPoolTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'GachaPool';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<GachaPool> get iterator => _rows.values.iterator;
  GachaPool? operator [](int key) => _rows[key];

  GachaPool get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `GachaPool` for key `$key`');
    }
    return row;
  }

  Map<int, GachaPool> get rows => _rows;
}

final class GachaItemTable extends Iterable<GachaItem> implements SoraConfigTable {
  final List<GachaItem> _rows;

  const GachaItemTable(
    this._rows,
  );

  static GachaItemTable decode(List<GachaItem> rows) {
    return GachaItemTable(
      rows,
    );
  }

  @override
  String get name => 'GachaItem';

  @override
  String get mode => 'list';

  @override
  String? get key => null;

  @override
  int get length => _rows.length;

  @override
  Iterator<GachaItem> get iterator => _rows.iterator;
  List<GachaItem> get rows => _rows;
}

final class EquipmentSetTable extends Iterable<EquipmentSet> implements SoraConfigTable {
  final Map<int, EquipmentSet> _rows;

  const EquipmentSetTable(
    this._rows,
  );

  static EquipmentSetTable decode(List<EquipmentSet> rows) {
    return EquipmentSetTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'EquipmentSet';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<EquipmentSet> get iterator => _rows.values.iterator;
  EquipmentSet? operator [](int key) => _rows[key];

  EquipmentSet get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `EquipmentSet` for key `$key`');
    }
    return row;
  }

  Map<int, EquipmentSet> get rows => _rows;
}

final class AchievementTable extends Iterable<Achievement> implements SoraConfigTable {
  final Map<int, Achievement> _rows;

  const AchievementTable(
    this._rows,
  );

  static AchievementTable decode(List<Achievement> rows) {
    return AchievementTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Achievement';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Achievement> get iterator => _rows.values.iterator;
  Achievement? operator [](int key) => _rows[key];

  Achievement get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Achievement` for key `$key`');
    }
    return row;
  }

  Map<int, Achievement> get rows => _rows;
}

final class VipLevelTable extends Iterable<VipLevel> implements SoraConfigTable {
  final Map<int, VipLevel> _rows;

  const VipLevelTable(
    this._rows,
  );

  static VipLevelTable decode(List<VipLevel> rows) {
    return VipLevelTable(
      decodeMapTable(rows, (row) => row.level),
    );
  }

  @override
  String get name => 'VipLevel';

  @override
  String get mode => 'map';

  @override
  String? get key => 'level';

  @override
  int get length => _rows.length;

  @override
  Iterator<VipLevel> get iterator => _rows.values.iterator;
  VipLevel? operator [](int key) => _rows[key];

  VipLevel get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `VipLevel` for key `$key`');
    }
    return row;
  }

  Map<int, VipLevel> get rows => _rows;
}

final class MailTemplateTable extends Iterable<MailTemplate> implements SoraConfigTable {
  final Map<int, MailTemplate> _rows;

  const MailTemplateTable(
    this._rows,
  );

  static MailTemplateTable decode(List<MailTemplate> rows) {
    return MailTemplateTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'MailTemplate';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<MailTemplate> get iterator => _rows.values.iterator;
  MailTemplate? operator [](int key) => _rows[key];

  MailTemplate get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `MailTemplate` for key `$key`');
    }
    return row;
  }

  Map<int, MailTemplate> get rows => _rows;
}

final class MailRewardTable extends Iterable<MailReward> implements SoraConfigTable {
  final List<MailReward> _rows;

  const MailRewardTable(
    this._rows,
  );

  static MailRewardTable decode(List<MailReward> rows) {
    return MailRewardTable(
      rows,
    );
  }

  @override
  String get name => 'MailReward';

  @override
  String get mode => 'list';

  @override
  String? get key => null;

  @override
  int get length => _rows.length;

  @override
  Iterator<MailReward> get iterator => _rows.iterator;
  List<MailReward> get rows => _rows;
}

final class DialogueTable extends Iterable<Dialogue> implements SoraConfigTable {
  final Map<int, Dialogue> _rows;

  const DialogueTable(
    this._rows,
  );

  static DialogueTable decode(List<Dialogue> rows) {
    return DialogueTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'Dialogue';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<Dialogue> get iterator => _rows.values.iterator;
  Dialogue? operator [](int key) => _rows[key];

  Dialogue get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `Dialogue` for key `$key`');
    }
    return row;
  }

  Map<int, Dialogue> get rows => _rows;
}

final class EventRuleTable extends Iterable<EventRule> implements SoraConfigTable {
  final Map<int, EventRule> _rows;

  const EventRuleTable(
    this._rows,
  );

  static EventRuleTable decode(List<EventRule> rows) {
    return EventRuleTable(
      decodeMapTable(rows, (row) => row.id),
    );
  }

  @override
  String get name => 'EventRule';

  @override
  String get mode => 'map';

  @override
  String? get key => 'id';

  @override
  int get length => _rows.length;

  @override
  Iterator<EventRule> get iterator => _rows.values.iterator;
  EventRule? operator [](int key) => _rows[key];

  EventRule get(int key) {
    final row = _rows[key];
    if (row == null) {
      throw SoraReadException('missing row in table `EventRule` for key `$key`');
    }
    return row;
  }

  Map<int, EventRule> get rows => _rows;
}

final class SoraConfig {
  final Map<Type, Object> _tables;

  const SoraConfig._(this._tables);

  static SoraConfig fromBytes(List<int> bytes) {
    final bundle = SoraValueBundle.parseJson(bytes);
    return SoraConfig._({
      ItemTable: ItemTable.decode(
        bundle.decodeTable('Item', Item.decode),
      ),
      SkillTable: SkillTable.decode(
        bundle.decodeTable('Skill', Skill.decode),
      ),
      QuestTable: QuestTable.decode(
        bundle.decodeTable('Quest', Quest.decode),
      ),
      QuestRewardTable: QuestRewardTable.decode(
        bundle.decodeTable('QuestReward', QuestReward.decode),
      ),
      GameSettingsTable: GameSettingsTable.decode(
        bundle.decodeTable('GameSettings', GameSettings.decode),
      ),
      LocalizationTable: LocalizationTable.decode(
        bundle.decodeTable('Localization', Localization.decode),
      ),
      LevelExpTable: LevelExpTable.decode(
        bundle.decodeTable('LevelExp', LevelExp.decode),
      ),
      CharacterTable: CharacterTable.decode(
        bundle.decodeTable('Character', Character.decode),
      ),
      CharacterSkillTable: CharacterSkillTable.decode(
        bundle.decodeTable('CharacterSkill', CharacterSkill.decode),
      ),
      BuffTable: BuffTable.decode(
        bundle.decodeTable('Buff', Buff.decode),
      ),
      DropGroupTable: DropGroupTable.decode(
        bundle.decodeTable('DropGroup', DropGroup.decode),
      ),
      DropEntryTable: DropEntryTable.decode(
        bundle.decodeTable('DropEntry', DropEntry.decode),
      ),
      MonsterTable: MonsterTable.decode(
        bundle.decodeTable('Monster', Monster.decode),
      ),
      StageTable: StageTable.decode(
        bundle.decodeTable('Stage', Stage.decode),
      ),
      StageRewardTable: StageRewardTable.decode(
        bundle.decodeTable('StageReward', StageReward.decode),
      ),
      DungeonTable: DungeonTable.decode(
        bundle.decodeTable('Dungeon', Dungeon.decode),
      ),
      ShopTable: ShopTable.decode(
        bundle.decodeTable('Shop', Shop.decode),
      ),
      ShopItemTable: ShopItemTable.decode(
        bundle.decodeTable('ShopItem', ShopItem.decode),
      ),
      RecipeTable: RecipeTable.decode(
        bundle.decodeTable('Recipe', Recipe.decode),
      ),
      GachaPoolTable: GachaPoolTable.decode(
        bundle.decodeTable('GachaPool', GachaPool.decode),
      ),
      GachaItemTable: GachaItemTable.decode(
        bundle.decodeTable('GachaItem', GachaItem.decode),
      ),
      EquipmentSetTable: EquipmentSetTable.decode(
        bundle.decodeTable('EquipmentSet', EquipmentSet.decode),
      ),
      AchievementTable: AchievementTable.decode(
        bundle.decodeTable('Achievement', Achievement.decode),
      ),
      VipLevelTable: VipLevelTable.decode(
        bundle.decodeTable('VipLevel', VipLevel.decode),
      ),
      MailTemplateTable: MailTemplateTable.decode(
        bundle.decodeTable('MailTemplate', MailTemplate.decode),
      ),
      MailRewardTable: MailRewardTable.decode(
        bundle.decodeTable('MailReward', MailReward.decode),
      ),
      DialogueTable: DialogueTable.decode(
        bundle.decodeTable('Dialogue', Dialogue.decode),
      ),
      EventRuleTable: EventRuleTable.decode(
        bundle.decodeTable('EventRule', EventRule.decode),
      ),
    });
  }

  Iterable<SoraConfigTable> get tables => _tables.values.cast<SoraConfigTable>();

  T _table<T extends Object>() {
    final table = _tables[T];
    if (table is T) {
      return table;
    }
    throw const SoraReadException('generated SoraConfig is missing a table or has an unexpected table type');
  }
  ItemTable get item => _table<ItemTable>();
  SkillTable get skill => _table<SkillTable>();
  QuestTable get quest => _table<QuestTable>();
  QuestRewardTable get questReward => _table<QuestRewardTable>();
  GameSettingsTable get gameSettings => _table<GameSettingsTable>();
  LocalizationTable get localization => _table<LocalizationTable>();
  LevelExpTable get levelExp => _table<LevelExpTable>();
  CharacterTable get character => _table<CharacterTable>();
  CharacterSkillTable get characterSkill => _table<CharacterSkillTable>();
  BuffTable get buff => _table<BuffTable>();
  DropGroupTable get dropGroup => _table<DropGroupTable>();
  DropEntryTable get dropEntry => _table<DropEntryTable>();
  MonsterTable get monster => _table<MonsterTable>();
  StageTable get stage => _table<StageTable>();
  StageRewardTable get stageReward => _table<StageRewardTable>();
  DungeonTable get dungeon => _table<DungeonTable>();
  ShopTable get shop => _table<ShopTable>();
  ShopItemTable get shopItem => _table<ShopItemTable>();
  RecipeTable get recipe => _table<RecipeTable>();
  GachaPoolTable get gachaPool => _table<GachaPoolTable>();
  GachaItemTable get gachaItem => _table<GachaItemTable>();
  EquipmentSetTable get equipmentSet => _table<EquipmentSetTable>();
  AchievementTable get achievement => _table<AchievementTable>();
  VipLevelTable get vipLevel => _table<VipLevelTable>();
  MailTemplateTable get mailTemplate => _table<MailTemplateTable>();
  MailRewardTable get mailReward => _table<MailRewardTable>();
  DialogueTable get dialogue => _table<DialogueTable>();
  EventRuleTable get eventRule => _table<EventRuleTable>();
}
