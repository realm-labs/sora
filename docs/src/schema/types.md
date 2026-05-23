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

| Property | Purpose |
| --- | --- |
| `required` | Requires a value unless a default applies. |
| `default` | Value used when the source cell is blank. |
| `key` | Marks the table key field. |
| `comment` | Description used in generated Excel headers. |
| `range` | Inclusive numeric range. |
| `length` | Inclusive collection or string length range. |
| `parser` | Cell parser hint such as `json`, `tuple`, `tuple_list`, or `map`. |
| `scope` | Includes the field only for selected generation/export scopes. |

Defaults are written as strings because they are parsed through the same type-aware conversion path as source data.
