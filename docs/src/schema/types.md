# Types

Sora type expressions are written as strings in schema fields.

## Primitive Types

| Type | Meaning |
| --- | --- |
| `bool` | Boolean value. |
| `i32` | 32-bit signed integer. |
| `i64` | 64-bit signed integer. |
| `f32` | 32-bit floating point value. |
| `f64` | 64-bit floating point value. |
| `string` | UTF-8 string. |

```toml
[[tables.fields]]
name = "level"
type = "i32"
required = true
range = [1, 100]
```

## Named Types

| Type | Example |
| --- | --- |
| Enum | `enum<ItemType>` |
| Struct | `struct<ResourceCost>` |
| Union | `union<RewardAction>` |
| Reference | `ref<Item.id>` |

```toml
[[tables.fields]]
name = "item_type"
type = "enum<ItemType>"
required = true

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

## Field Rules

The same field object is used in `[[tables.fields]]`, `[[structs.fields]]`, and `[[unions.variants.fields]]`. Table-only settings are ignored or invalid outside table fields as noted below.

| Property | Applies To | Purpose |
| --- | --- | --- |
| `name` | all fields | Field name used in source data, validation errors, generated code, and exported runtime data. |
| `type` | all fields | Type expression such as `i32`, `struct<ResourceCost>`, or `list<union<RewardAction>>`. |
| `required` | all fields | Requires a value unless a default applies. Defaults to `false`. |
| `default` | all fields except aggregation fields | String value used when the source cell or object field is absent. |
| `key` | table fields | Marks the table key field. Usually matches the table-level `key`. |
| `comment` | all fields | Description used in generated Excel headers. |
| `range` | numeric fields and numeric collection elements | Inclusive numeric range, written as `[min, max]`. |
| `length` | `string`, `list`, `set`, `array`, `map` | Inclusive length range, written as `[min, max]`. |
| `parser` | cell-based inputs and defaults | Cell parser hint. See [Cell Parsers](parsers.md). |
| `scope` | all fields | Includes the field only for selected generation/export scopes. Defaults to `all`. |
| `source_table` | table fields only | Aggregation source table. Must be used with `parent_key` and `child_key`. |
| `parent_key` | table fields only | Aggregation key field on the owner table. |
| `child_key` | table fields only | Aggregation key field on the source table. |
| `value_field` | table aggregation fields | Optional source-table field copied as the aggregation value. |
| `order_by` | table aggregation fields | Optional source-table field used for ascending aggregation order. |

Defaults are written as strings because they are parsed through the same type-aware conversion path as source data.

`source_table`, `parent_key`, and `child_key` describe aggregation fields; see [References and Aggregation](references.md). Aggregation fields can be `list<T>`, `T`, or `optional<T>` and cannot declare `default`.
