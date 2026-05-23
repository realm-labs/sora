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

References must point to the primary key of a `mode = "map"` table. Containers can wrap references, for example `list<ref<Item.id>>`.

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

`[[tables.fields]]`, `[[structs.fields]]`, and `[[unions.variants.fields]]` share the common field properties. Table fields have extra table-only properties for keys and derived values; those properties are invalid on struct fields and union variant fields.

| Property | Applies To | Purpose |
| --- | --- | --- |
| `name` | all fields | Field name used in source data, validation errors, generated code, and exported runtime data. |
| `type` | all fields | Type expression such as `i32`, `struct<ResourceCost>`, or `list<union<RewardAction>>`. |
| `required` | all fields | Requires a value unless a default applies. Defaults to `false`. |
| `default` | all fields except derived fields | String value used when the source cell or object field is absent. |
| `key` | table fields only | Marks the table key field. Usually matches the table-level `key`. |
| `comment` | all fields | Description used in generated Excel headers. |
| `range` | numeric fields and numeric collection elements | Inclusive numeric range, written as `[min, max]`. |
| `length` | `string`, `list`, `set`, `array`, `map` | Inclusive length range, written as `[min, max]`. |
| `parser` | cell-based inputs and defaults | Cell parser hint. See [Cell Parsers](parsers.md). |
| `scope` | all fields | Includes the field only for selected generation/export scopes. Defaults to `all`. |
| `from` | table fields only | Optional child-table source for a derived field. |

Defaults are written as strings because they are parsed through the same type-aware conversion path as source data.

`from` describes a field derived from matching rows in another table; see [References and Derived Fields](references.md). Derived fields can be `list<T>`, `T`, or `optional<T>` and cannot declare `default`.
