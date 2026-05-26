# Types

Sora type expressions are written as strings in schema fields.

## Primitive Types

| Type | Meaning |
| --- | --- |
| `bool` | Boolean value. |
| `i8` | 8-bit signed integer. |
| `u8` | 8-bit unsigned integer. |
| `i16` | 16-bit signed integer. |
| `u16` | 16-bit unsigned integer. |
| `i32` | 32-bit signed integer. |
| `u32` | 32-bit unsigned integer. |
| `i64` | 64-bit signed integer. |
| `f32` | 32-bit floating point value. |
| `f64` | 64-bit floating point value. |
| `string` | UTF-8 string. |
| `text` | Localization text key. See [Localization](../localization.md). |

Integer widths are validated by Sora before export. Some target languages do not have unsigned small integer types, so generated code may use a wider signed type while preserving the schema range.

```toml
[[tables.fields]]
name = "level"
type = "u16"
range = [1, 100]
```

## Named Types

| Type | Example |
| --- | --- |
| Enum | `enum<ItemType>` |
| Struct | `struct<ResourceCost>` |
| Union | `union<RewardAction>` |
| Reference | `ref<Item.id>` |

References must point to the primary key of a `mode = "map"` table. Containers can wrap references, for example `list<ref<Item.id>>`.

```toml
[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"

[[tables.fields]]
name = "price"
type = "struct<ResourceCost>"
parser = { kind = "tuple" }
```

## Collections

| Type | Meaning |
| --- | --- |
| `list<T>` | Ordered repeated values. |
| `set<T>` | Unique repeated values. |
| `array<T,N>` | Fixed-length repeated values. |
| `map<K,V>` | Key/value pairs. |
| `optional<T>` | Nullable or absent value. |

```toml
[[tables.fields]]
name = "tags"
type = "set<string>"
parser = { kind = "json" }
default = "[\"misc\"]"

[[tables.fields]]
name = "attributes"
type = "map<string,i32>"
parser = { kind = "map" }
```

## Cell Examples

These examples show what a designer would put in an Excel or CSV cell:

| Field type | Parser | Cell value |
| --- | --- | --- |
| `u16` | none | `1001` |
| `enum<ItemType>` | none | `Weapon` |
| `list<i32>` | none or `split` | `1,2,3` |
| `text` | none | `quest.1001.title` |
| `set<string>` | `json` | `["starter","melee"]` |
| `struct<ResourceCost>` | `tuple` | `Gold,0,100` |
| `struct<ResourceCost>` | `columns` | spread across `cost_kind`, `cost_id`, `cost_count` columns |
| `map<string,i32>` | `map` | `atk,10\|hp,20` |
| `union<EventCondition>` | `json` | `{"type":"QuestCompleted","quest_id":5002}` |
| `optional<ref<Item.id>>` | none | empty cell or `1001` |

## Field Rules

`[[tables.fields]]`, `[[structs.fields]]`, and `[[unions.variants.fields]]` share the common field properties. Table fields have extra table-only properties for derived values; those properties are invalid on struct fields and union variant fields. A table primary key is declared once on the table itself with `key = "field_name"`.

Field presence is part of the type: `optional<T>` means the value may be absent or null, while every other type is required unless a `default` fills the missing value.

For TOML/JSON/YAML-style object inputs, a field can be absent from the object. For Excel and CSV, the column must exist in the header; an omitted cell, blank cell, or short CSV record is treated as an empty cell.

| Schema field | Object field absent | Excel/CSV cell empty |
| --- | --- | --- |
| `type = "i32"` | Validation error. | Validation error. |
| `type = "optional<i32>"` | `null`. | `null`. |
| `type = "i32"` plus `default = "1"` | `1`. | `1`. |
| `type = "optional<i32>"` plus `default = "1"` | `1`. | `null`. |

| Property | Applies To | Purpose |
| --- | --- | --- |
| `name` | all fields | Field name used in source data, validation errors, generated code, and exported runtime data. |
| `type` | all fields | Type expression such as `i32`, `struct<ResourceCost>`, or `list<union<RewardAction>>`. |
| `default` | all fields except derived fields | String value used when the source object field is absent or a required Excel/CSV cell is empty. |
| `comment` | all fields | Description used in generated Excel headers. |
| `range` | numeric fields and numeric collection elements | Inclusive numeric range, written as `[min, max]`. |
| `length` | `string`, `list`, `set`, `array`, `map` | Inclusive length range, written as `[min, max]`. |
| `parser` | cell-based inputs and defaults | Cell parser hint. See [Cell Parsers](parsers.md). |
| `scope` | all fields | Includes the field only for selected generation/export scopes. Defaults to `all`. |
| `from` | table fields only | Optional child-table source for a derived field. |

Defaults are written as strings because they are parsed through the same type-aware conversion path as source data.

`from` describes a field derived from matching rows in another table; see [References and Derived Fields](references.md). Derived fields can be `list<T>`, `T`, or `optional<T>` and cannot declare `default`.
